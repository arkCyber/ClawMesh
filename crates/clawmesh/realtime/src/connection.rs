//! WebSocket connection handling
//! 
//! This module provides WebSocket connection management without
//! depending on actix_web_actors.

use crate::RealtimeConfig;
use tracing::info;

/// WebSocket connection handler
pub struct WsConnectionHandler {
    config: RealtimeConfig,
}

impl WsConnectionHandler {
    /// Create a new connection handler
    #[must_use]
    pub fn new(config: RealtimeConfig) -> Self {
        Self { config }
    }
    
    /// Get the configuration
    pub fn config(&self) -> &RealtimeConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_handler_creation() {
        let config = RealtimeConfig::default();
        let handler = WsConnectionHandler::new(config);
        assert_eq!(handler.config().heartbeat_interval, 30);
    }
}
