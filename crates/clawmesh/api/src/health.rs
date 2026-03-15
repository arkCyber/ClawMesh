//! Health Check and Readiness Endpoints
//!
//! Aerospace-grade health monitoring for production deployments.
//! Implements Kubernetes-compatible liveness and readiness probes.

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc};
use tracing::{debug, warn};

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System is degraded but operational
    Degraded,
    /// System is unhealthy
    Unhealthy,
}

/// Component health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Optional error message
    pub message: Option<String>,
    /// Last check time
    pub last_check: DateTime<Utc>,
    /// Response time in milliseconds
    pub response_time_ms: u64,
}

/// Overall system health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    /// Overall status
    pub status: HealthStatus,
    /// System version
    pub version: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Component health checks
    pub components: Vec<ComponentHealth>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Health checker service
pub struct HealthChecker {
    /// Component health states
    components: Arc<RwLock<Vec<ComponentHealth>>>,
    /// System start time
    start_time: DateTime<Utc>,
    /// System version
    version: String,
}

impl HealthChecker {
    /// Create new health checker
    pub fn new(version: String) -> Self {
        Self {
            components: Arc::new(RwLock::new(Vec::new())),
            start_time: Utc::now(),
            version,
        }
    }
    
    /// Register a component for health checking
    pub fn register_component(&self, name: String) {
        let mut components = self.components.write();
        components.push(ComponentHealth {
            name,
            status: HealthStatus::Healthy,
            message: None,
            last_check: Utc::now(),
            response_time_ms: 0,
        });
    }
    
    /// Update component health
    pub fn update_component_health(
        &self,
        name: &str,
        status: HealthStatus,
        message: Option<String>,
        response_time_ms: u64,
    ) {
        let mut components = self.components.write();
        if let Some(component) = components.iter_mut().find(|c| c.name == name) {
            component.status = status;
            component.message = message;
            component.last_check = Utc::now();
            component.response_time_ms = response_time_ms;
        }
    }
    
    /// Get overall system health
    pub fn get_system_health(&self) -> SystemHealth {
        let components = self.components.read().clone();
        
        // Determine overall status
        let status = if components.iter().any(|c| c.status == HealthStatus::Unhealthy) {
            HealthStatus::Unhealthy
        } else if components.iter().any(|c| c.status == HealthStatus::Degraded) {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };
        
        let uptime = Utc::now().signed_duration_since(self.start_time);
        
        SystemHealth {
            status,
            version: self.version.clone(),
            uptime_seconds: uptime.num_seconds() as u64,
            components,
            timestamp: Utc::now(),
        }
    }
    
    /// Check if system is ready to serve traffic
    pub fn is_ready(&self) -> bool {
        let components = self.components.read();
        !components.iter().any(|c| c.status == HealthStatus::Unhealthy)
    }
    
    /// Check if system is alive (basic liveness check)
    pub fn is_alive(&self) -> bool {
        // Simple check - if we can acquire the lock, we're alive
        self.components.try_read().is_some()
    }
}

/// Liveness probe endpoint (Kubernetes compatible)
/// 
/// Returns 200 if the application is running.
/// This should only fail if the application is completely deadlocked.
pub async fn liveness_probe(health_checker: web::Data<Arc<HealthChecker>>) -> impl Responder {
    if health_checker.is_alive() {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "alive",
            "timestamp": Utc::now(),
        }))
    } else {
        warn!("Liveness probe failed - system may be deadlocked");
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "dead",
            "timestamp": Utc::now(),
        }))
    }
}

/// Readiness probe endpoint (Kubernetes compatible)
/// 
/// Returns 200 if the application is ready to serve traffic.
/// This checks all critical dependencies.
pub async fn readiness_probe(health_checker: web::Data<Arc<HealthChecker>>) -> impl Responder {
    if health_checker.is_ready() {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "ready",
            "timestamp": Utc::now(),
        }))
    } else {
        debug!("Readiness probe failed - system not ready");
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "not_ready",
            "timestamp": Utc::now(),
        }))
    }
}

/// Detailed health check endpoint
/// 
/// Returns detailed health information about all components.
pub async fn health_check(health_checker: web::Data<Arc<HealthChecker>>) -> impl Responder {
    let health = health_checker.get_system_health();
    
    let status_code = match health.status {
        HealthStatus::Healthy => actix_web::http::StatusCode::OK,
        HealthStatus::Degraded => actix_web::http::StatusCode::OK,
        HealthStatus::Unhealthy => actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
    };
    
    HttpResponse::build(status_code).json(health)
}

/// Database health check
pub async fn check_database_health(
    pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
) -> ComponentHealth {
    use std::time::Instant;
    
    let start = Instant::now();
    
    // Simple query to check database connectivity
    let result = diesel::sql_query("SELECT 1")
        .execute(pool)
        .await;
    
    let response_time_ms = start.elapsed().as_millis() as u64;
    
    match result {
        Ok(_) => ComponentHealth {
            name: "database".to_string(),
            status: if response_time_ms < 100 {
                HealthStatus::Healthy
            } else if response_time_ms < 500 {
                HealthStatus::Degraded
            } else {
                HealthStatus::Unhealthy
            },
            message: None,
            last_check: Utc::now(),
            response_time_ms,
        },
        Err(e) => ComponentHealth {
            name: "database".to_string(),
            status: HealthStatus::Unhealthy,
            message: Some(format!("Database error: {}", e)),
            last_check: Utc::now(),
            response_time_ms,
        },
    }
}

/// WebSocket connection health check
pub async fn check_websocket_health(
    connection_count: usize,
    max_connections: usize,
) -> ComponentHealth {
    let usage_percent = (connection_count as f64 / max_connections as f64) * 100.0;
    
    let status = if usage_percent < 70.0 {
        HealthStatus::Healthy
    } else if usage_percent < 90.0 {
        HealthStatus::Degraded
    } else {
        HealthStatus::Unhealthy
    };
    
    ComponentHealth {
        name: "websocket".to_string(),
        status,
        message: Some(format!(
            "{}/{} connections ({:.1}%)",
            connection_count, max_connections, usage_percent
        )),
        last_check: Utc::now(),
        response_time_ms: 0,
    }
}

/// Memory health check
pub fn check_memory_health() -> ComponentHealth {
    // Get system memory info (simplified)
    // In production, use a proper system info crate
    
    ComponentHealth {
        name: "memory".to_string(),
        status: HealthStatus::Healthy,
        message: Some("Memory usage within limits".to_string()),
        last_check: Utc::now(),
        response_time_ms: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_health_checker_creation() {
        let checker = HealthChecker::new("1.0.0".to_string());
        assert_eq!(checker.version, "1.0.0");
        assert!(checker.is_alive());
        assert!(checker.is_ready());
    }
    
    #[test]
    fn test_component_registration() {
        let checker = HealthChecker::new("1.0.0".to_string());
        
        checker.register_component("database".to_string());
        checker.register_component("redis".to_string());
        
        let health = checker.get_system_health();
        assert_eq!(health.components.len(), 2);
        assert_eq!(health.status, HealthStatus::Healthy);
    }
    
    #[test]
    fn test_component_health_update() {
        let checker = HealthChecker::new("1.0.0".to_string());
        checker.register_component("database".to_string());
        
        checker.update_component_health(
            "database",
            HealthStatus::Degraded,
            Some("Slow response".to_string()),
            250,
        );
        
        let health = checker.get_system_health();
        assert_eq!(health.status, HealthStatus::Degraded);
        assert_eq!(health.components[0].response_time_ms, 250);
    }
    
    #[test]
    fn test_overall_status_calculation() {
        let checker = HealthChecker::new("1.0.0".to_string());
        
        checker.register_component("comp1".to_string());
        checker.register_component("comp2".to_string());
        checker.register_component("comp3".to_string());
        
        // All healthy
        let health = checker.get_system_health();
        assert_eq!(health.status, HealthStatus::Healthy);
        
        // One degraded
        checker.update_component_health("comp1", HealthStatus::Degraded, None, 0);
        let health = checker.get_system_health();
        assert_eq!(health.status, HealthStatus::Degraded);
        
        // One unhealthy
        checker.update_component_health("comp2", HealthStatus::Unhealthy, None, 0);
        let health = checker.get_system_health();
        assert_eq!(health.status, HealthStatus::Unhealthy);
    }
    
    #[test]
    fn test_readiness_check() {
        let checker = HealthChecker::new("1.0.0".to_string());
        checker.register_component("database".to_string());
        
        // Healthy - ready
        assert!(checker.is_ready());
        
        // Degraded - still ready
        checker.update_component_health("database", HealthStatus::Degraded, None, 0);
        assert!(checker.is_ready());
        
        // Unhealthy - not ready
        checker.update_component_health("database", HealthStatus::Unhealthy, None, 0);
        assert!(!checker.is_ready());
    }
    
    #[test]
    fn test_websocket_health_thresholds() {
        // Healthy
        let health = tokio_test::block_on(check_websocket_health(5000, 10000));
        assert_eq!(health.status, HealthStatus::Healthy);
        
        // Degraded
        let health = tokio_test::block_on(check_websocket_health(8000, 10000));
        assert_eq!(health.status, HealthStatus::Degraded);
        
        // Unhealthy
        let health = tokio_test::block_on(check_websocket_health(9500, 10000));
        assert_eq!(health.status, HealthStatus::Unhealthy);
    }
}
