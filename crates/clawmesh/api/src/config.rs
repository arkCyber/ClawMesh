//! Configuration Management
//!
//! Aerospace-grade configuration with validation and environment support.
//! Ensures all configuration is validated before application startup.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// Database configuration
    pub database: DatabaseConfig,
    /// WebSocket configuration
    pub websocket: WebSocketConfig,
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Worker threads
    pub workers: usize,
    /// Max blocking threads
    pub max_blocking_threads: usize,
    /// Request timeout in seconds
    pub request_timeout: u64,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL
    pub url: String,
    /// Maximum connections
    pub max_connections: u32,
    /// Minimum connections
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Idle timeout in seconds
    pub idle_timeout: u64,
    /// Max connection lifetime in seconds
    pub max_lifetime: u64,
}

/// WebSocket configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Number of connection shards
    pub shard_count: usize,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Max requests per minute for authenticated users
    pub max_requests_per_user: u32,
    /// Max requests per minute per IP
    pub max_requests_per_ip: u32,
    /// Burst allowance
    pub burst_allowance: u32,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable Prometheus metrics
    pub prometheus_enabled: bool,
    /// Metrics endpoint path
    pub metrics_path: String,
    /// Health check endpoint path
    pub health_path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            websocket: WebSocketConfig::default(),
            rate_limit: RateLimitConfig::default(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: num_cpus::get(),
            max_blocking_threads: 512,
            request_timeout: 30,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://localhost/clawmesh".to_string(),
            max_connections: 100,
            min_connections: 10,
            connection_timeout: 5,
            idle_timeout: 600,
            max_lifetime: 1800,
        }
    }
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            max_connections: 150_000,
            heartbeat_interval: 30,
            connection_timeout: 60,
            shard_count: 256,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_requests_per_user: 100,
            max_requests_per_ip: 1000,
            burst_allowance: 20,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            prometheus_enabled: true,
            metrics_path: "/metrics".to_string(),
            health_path: "/health".to_string(),
        }
    }
}

impl AppConfig {
    /// Load configuration from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::FileRead(e.to_string()))?;
        
        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| ConfigError::Parse(e.to_string()))?;
        
        config.validate()?;
        
        Ok(config)
    }
    
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = AppConfig::default();
        
        // Server
        if let Ok(host) = std::env::var("SERVER_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = std::env::var("SERVER_PORT") {
            config.server.port = port.parse()
                .map_err(|_| ConfigError::InvalidValue("SERVER_PORT".to_string()))?;
        }
        
        // Database
        if let Ok(url) = std::env::var("DATABASE_URL") {
            config.database.url = url;
        }
        if let Ok(max_conn) = std::env::var("DATABASE_MAX_CONNECTIONS") {
            config.database.max_connections = max_conn.parse()
                .map_err(|_| ConfigError::InvalidValue("DATABASE_MAX_CONNECTIONS".to_string()))?;
        }
        
        // WebSocket
        if let Ok(max_conn) = std::env::var("WEBSOCKET_MAX_CONNECTIONS") {
            config.websocket.max_connections = max_conn.parse()
                .map_err(|_| ConfigError::InvalidValue("WEBSOCKET_MAX_CONNECTIONS".to_string()))?;
        }
        
        config.validate()?;
        
        Ok(config)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate server
        if self.server.port == 0 {
            return Err(ConfigError::InvalidValue("server.port cannot be 0".to_string()));
        }
        if self.server.workers == 0 {
            return Err(ConfigError::InvalidValue("server.workers cannot be 0".to_string()));
        }
        
        // Validate database
        if self.database.max_connections == 0 {
            return Err(ConfigError::InvalidValue("database.max_connections cannot be 0".to_string()));
        }
        if self.database.min_connections > self.database.max_connections {
            return Err(ConfigError::InvalidValue(
                "database.min_connections cannot exceed max_connections".to_string()
            ));
        }
        
        // Validate WebSocket
        if self.websocket.max_connections == 0 {
            return Err(ConfigError::InvalidValue("websocket.max_connections cannot be 0".to_string()));
        }
        if self.websocket.shard_count == 0 {
            return Err(ConfigError::InvalidValue("websocket.shard_count cannot be 0".to_string()));
        }
        if !self.websocket.shard_count.is_power_of_two() {
            return Err(ConfigError::InvalidValue(
                "websocket.shard_count should be a power of 2 for optimal performance".to_string()
            ));
        }
        
        // Validate rate limiting
        if self.rate_limit.enabled {
            if self.rate_limit.max_requests_per_user == 0 {
                return Err(ConfigError::InvalidValue(
                    "rate_limit.max_requests_per_user cannot be 0".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::Serialize(e.to_string()))?;
        
        fs::write(path, content)
            .map_err(|e| ConfigError::FileWrite(e.to_string()))?;
        
        Ok(())
    }
}

/// Configuration error
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileRead(String),
    
    #[error("Failed to write config file: {0}")]
    FileWrite(String),
    
    #[error("Failed to parse config: {0}")]
    Parse(String),
    
    #[error("Failed to serialize config: {0}")]
    Serialize(String),
    
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.port, 8080);
        assert!(config.rate_limit.enabled);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        
        // Valid config
        assert!(config.validate().is_ok());
        
        // Invalid port
        config.server.port = 0;
        assert!(config.validate().is_err());
        config.server.port = 8080;
        
        // Invalid workers
        config.server.workers = 0;
        assert!(config.validate().is_err());
        config.server.workers = 4;
        
        // Invalid database connections
        config.database.min_connections = 100;
        config.database.max_connections = 10;
        assert!(config.validate().is_err());
        config.database.min_connections = 10;
        config.database.max_connections = 100;
        
        // Invalid shard count (not power of 2)
        config.websocket.shard_count = 100;
        assert!(config.validate().is_err());
        config.websocket.shard_count = 256;
        
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        
        assert!(toml_str.contains("[server]"));
        assert!(toml_str.contains("[database]"));
        assert!(toml_str.contains("[websocket]"));
    }
    
    #[test]
    fn test_shard_count_power_of_two() {
        let mut config = AppConfig::default();
        
        // Valid power of 2
        config.websocket.shard_count = 128;
        assert!(config.validate().is_ok());
        
        config.websocket.shard_count = 256;
        assert!(config.validate().is_ok());
        
        config.websocket.shard_count = 512;
        assert!(config.validate().is_ok());
        
        // Invalid (not power of 2)
        config.websocket.shard_count = 100;
        assert!(config.validate().is_err());
        
        config.websocket.shard_count = 200;
        assert!(config.validate().is_err());
    }
}
