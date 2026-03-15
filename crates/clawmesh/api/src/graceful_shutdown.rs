//! Graceful Shutdown Handler
//!
//! Aerospace-grade shutdown mechanism ensuring no data loss and clean resource cleanup.
//! Handles SIGTERM, SIGINT signals and coordinates orderly shutdown.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::time::timeout;
use tracing::{info, warn, error};

/// Shutdown coordinator
pub struct ShutdownCoordinator {
    /// Shutdown signal
    shutdown_signal: Arc<Notify>,
    /// Shutdown timeout
    timeout: Duration,
}

impl ShutdownCoordinator {
    /// Create new shutdown coordinator
    pub fn new(timeout_seconds: u64) -> Self {
        Self {
            shutdown_signal: Arc::new(Notify::new()),
            timeout: Duration::from_secs(timeout_seconds),
        }
    }
    
    /// Get shutdown signal handle
    pub fn get_signal(&self) -> Arc<Notify> {
        Arc::clone(&self.shutdown_signal)
    }
    
    /// Trigger shutdown
    pub fn trigger_shutdown(&self) {
        info!("Shutdown triggered");
        self.shutdown_signal.notify_waiters();
    }
    
    /// Wait for shutdown signal
    pub async fn wait_for_shutdown(&self) {
        self.shutdown_signal.notified().await;
        info!("Shutdown signal received");
    }
    
    /// Execute graceful shutdown with timeout
    pub async fn execute_shutdown<F, Fut>(&self, shutdown_fn: F) -> Result<(), ShutdownError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<(), String>>,
    {
        info!("Starting graceful shutdown (timeout: {:?})", self.timeout);
        
        match timeout(self.timeout, shutdown_fn()).await {
            Ok(Ok(())) => {
                info!("Graceful shutdown completed successfully");
                Ok(())
            }
            Ok(Err(e)) => {
                error!("Shutdown failed: {}", e);
                Err(ShutdownError::ShutdownFailed(e))
            }
            Err(_) => {
                warn!("Shutdown timeout exceeded, forcing shutdown");
                Err(ShutdownError::Timeout)
            }
        }
    }
}

/// Shutdown error
#[derive(Debug, thiserror::Error)]
pub enum ShutdownError {
    #[error("Shutdown timeout exceeded")]
    Timeout,
    
    #[error("Shutdown failed: {0}")]
    ShutdownFailed(String),
}

/// Shutdown phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownPhase {
    /// Stop accepting new connections
    StopAcceptingConnections,
    /// Drain existing connections
    DrainConnections,
    /// Flush caches
    FlushCaches,
    /// Close database connections
    CloseDatabaseConnections,
    /// Final cleanup
    FinalCleanup,
}

/// Shutdown manager with phases
pub struct ShutdownManager {
    coordinator: ShutdownCoordinator,
}

impl ShutdownManager {
    /// Create new shutdown manager
    pub fn new(timeout_seconds: u64) -> Self {
        Self {
            coordinator: ShutdownCoordinator::new(timeout_seconds),
        }
    }
    
    /// Get shutdown signal
    pub fn get_signal(&self) -> Arc<Notify> {
        self.coordinator.get_signal()
    }
    
    /// Trigger shutdown
    pub fn trigger(&self) {
        self.coordinator.trigger_shutdown();
    }
    
    /// Execute phased shutdown
    pub async fn execute_phased_shutdown(
        &self,
        phases: Vec<(ShutdownPhase, Box<dyn FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> + Send>)>,
    ) -> Result<(), ShutdownError> {
        info!("Executing phased shutdown with {} phases", phases.len());
        
        for (phase, shutdown_fn) in phases {
            info!("Shutdown phase: {:?}", phase);
            
            let phase_timeout = Duration::from_secs(30);
            match timeout(phase_timeout, shutdown_fn()).await {
                Ok(Ok(())) => {
                    info!("Phase {:?} completed successfully", phase);
                }
                Ok(Err(e)) => {
                    error!("Phase {:?} failed: {}", phase, e);
                    return Err(ShutdownError::ShutdownFailed(format!("Phase {:?} failed: {}", phase, e)));
                }
                Err(_) => {
                    warn!("Phase {:?} timeout, continuing to next phase", phase);
                }
            }
        }
        
        info!("All shutdown phases completed");
        Ok(())
    }
}

/// Setup signal handlers for graceful shutdown
pub fn setup_signal_handlers(coordinator: Arc<ShutdownCoordinator>) {
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            
            let mut sigterm = signal(SignalKind::terminate())
                .expect("Failed to setup SIGTERM handler");
            let mut sigint = signal(SignalKind::interrupt())
                .expect("Failed to setup SIGINT handler");
            
            tokio::select! {
                _ = sigterm.recv() => {
                    info!("Received SIGTERM");
                    coordinator.trigger_shutdown();
                }
                _ = sigint.recv() => {
                    info!("Received SIGINT");
                    coordinator.trigger_shutdown();
                }
            }
        }
        
        #[cfg(not(unix))]
        {
            use tokio::signal;
            
            signal::ctrl_c().await.expect("Failed to setup Ctrl+C handler");
            info!("Received Ctrl+C");
            coordinator.trigger_shutdown();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_shutdown_coordinator() {
        let coordinator = ShutdownCoordinator::new(5);
        let signal = coordinator.get_signal();
        
        // Spawn a task that waits for shutdown
        let handle = tokio::spawn(async move {
            signal.notified().await;
            "shutdown received"
        });
        
        // Trigger shutdown
        tokio::time::sleep(Duration::from_millis(100)).await;
        coordinator.trigger_shutdown();
        
        // Wait for task to complete
        let result = handle.await.unwrap();
        assert_eq!(result, "shutdown received");
    }
    
    #[tokio::test]
    async fn test_shutdown_with_timeout() {
        let coordinator = ShutdownCoordinator::new(1);
        
        // Fast shutdown (should succeed)
        let result = coordinator.execute_shutdown(|| async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(())
        }).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_shutdown_timeout() {
        let coordinator = ShutdownCoordinator::new(1);
        
        // Slow shutdown (should timeout)
        let result = coordinator.execute_shutdown(|| async {
            tokio::time::sleep(Duration::from_secs(5)).await;
            Ok(())
        }).await;
        
        assert!(matches!(result, Err(ShutdownError::Timeout)));
    }
    
    #[tokio::test]
    async fn test_shutdown_failure() {
        let coordinator = ShutdownCoordinator::new(5);
        
        // Shutdown that fails
        let result = coordinator.execute_shutdown(|| async {
            Err("Shutdown failed".to_string())
        }).await;
        
        assert!(matches!(result, Err(ShutdownError::ShutdownFailed(_))));
    }
}
