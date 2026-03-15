//! WebSocket session management
//!
//! This module provides WebSocket session management without
//! depending on actix_web_actors.

use crate::messages::{ClientMessage, MessageType, ServerMessage};
use crate::ConnectionId;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// WebSocket session state
pub struct WsSession {
    /// Unique connection ID
    pub id: ConnectionId,
    /// User ID (if authenticated)
    pub user_id: Option<i32>,
    /// Last heartbeat time
    pub heartbeat: Instant,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// Connection timeout
    pub timeout: Duration,
}

impl WsSession {
    /// Create a new WebSocket session
    #[must_use]
    pub fn new(heartbeat_interval: u64, timeout: u64) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            user_id: None,
            heartbeat: Instant::now(),
            heartbeat_interval: Duration::from_secs(heartbeat_interval),
            timeout: Duration::from_secs(timeout),
        }
    }

    /// Check if the session has timed out
    pub fn is_timed_out(&self) -> bool {
        Instant::now().duration_since(self.heartbeat) > self.timeout
    }

    /// Update heartbeat timestamp
    pub fn update_heartbeat(&mut self) {
        self.heartbeat = Instant::now();
    }

    /// Handle incoming client message
    pub fn handle_client_message(&mut self, msg: ClientMessage) -> Option<ServerMessage> {
        match msg.msg_type {
            MessageType::Ping => {
                Some(ServerMessage::pong())
            }
            MessageType::Text => {
                debug!("Received text message from connection {}", self.id);
                None
            }
            MessageType::Join => {
                if let Some(room_id) = msg.room_id {
                    info!("Connection {} joining room {}", self.id, room_id);
                }
                None
            }
            MessageType::Leave => {
                if let Some(room_id) = msg.room_id {
                    info!("Connection {} leaving room {}", self.id, room_id);
                }
                None
            }
            _ => {
                warn!("Unhandled message type: {:?}", msg.msg_type);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = WsSession::new(30, 300);
        assert!(session.user_id.is_none());
        assert_eq!(session.heartbeat_interval, Duration::from_secs(30));
        assert_eq!(session.timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_heartbeat_update() {
        let mut session = WsSession::new(30, 300);
        let old_heartbeat = session.heartbeat;
        std::thread::sleep(Duration::from_millis(10));
        session.update_heartbeat();
        assert!(session.heartbeat > old_heartbeat);
    }
}
