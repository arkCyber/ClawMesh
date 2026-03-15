//! ClawMesh Real-time Communication System
//! 
//! Provides WebSocket-based real-time communication capabilities including:
//! - Real-time messaging
//! - Live notifications
//! - Online presence tracking
//! - Event broadcasting

pub mod connection;
pub mod manager;
pub mod messages;
pub mod session;

pub use connection::WsConnectionHandler;
pub use manager::{ConnectionManager, RoomManager};
pub use messages::{ClientMessage, ServerMessage, MessageType};
pub use session::WsSession;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Connection ID type
pub type ConnectionId = Uuid;

/// Room ID type
pub type RoomId = String;

/// User presence status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PresenceStatus {
    /// User is online and active
    Online,
    /// User is away
    Away,
    /// User is busy
    Busy,
    /// User is offline
    Offline,
}

/// User presence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresence {
    /// User ID
    pub user_id: i32,
    /// Current status
    pub status: PresenceStatus,
    /// Last seen timestamp
    pub last_seen: chrono::DateTime<chrono::Utc>,
    /// Current connection IDs
    pub connections: Vec<ConnectionId>,
}

/// Real-time configuration
#[derive(Debug, Clone)]
pub struct RealtimeConfig {
    /// Maximum connections per user
    pub max_connections_per_user: usize,
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Maximum message size in bytes
    pub max_message_size: usize,
}

impl Default for RealtimeConfig {
    fn default() -> Self {
        Self {
            max_connections_per_user: 5,
            heartbeat_interval: 30,
            connection_timeout: 300,
            max_message_size: 65_536,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_status() {
        let status = PresenceStatus::Online;
        assert_eq!(status, PresenceStatus::Online);
    }

    #[test]
    fn test_default_config() {
        let config = RealtimeConfig::default();
        assert_eq!(config.max_connections_per_user, 5);
        assert_eq!(config.heartbeat_interval, 30);
        assert_eq!(config.connection_timeout, 300);
    }
}
