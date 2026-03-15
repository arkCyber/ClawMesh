//! P2P Signaling Server
//!
//! WebSocket-based signaling for P2P connection negotiation.
//! Handles peer discovery, connection offers, and ICE candidate exchange.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tracing::{debug, info, warn, instrument};
use chrono::{DateTime, Utc};

/// Signaling message type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SignalingMessage {
    /// Offer to establish P2P connection
    Offer {
        from: i32,
        to: i32,
        transfer_id: String,
        sdp: String,
    },
    /// Answer to connection offer
    Answer {
        from: i32,
        to: i32,
        transfer_id: String,
        sdp: String,
    },
    /// ICE candidate for NAT traversal
    IceCandidate {
        from: i32,
        to: i32,
        transfer_id: String,
        candidate: String,
    },
    /// Connection established
    Connected {
        transfer_id: String,
        peer_id: i32,
    },
    /// Connection failed
    Failed {
        transfer_id: String,
        reason: String,
    },
    /// Heartbeat
    Ping,
    /// Heartbeat response
    Pong,
}

/// WebSocket session
#[derive(Debug, Clone)]
pub struct WebSocketSession {
    pub user_id: i32,
    pub session_id: String,
    pub connected_at: DateTime<Utc>,
    pub last_ping: DateTime<Utc>,
}

/// Signaling server
pub struct SignalingServer {
    /// Active sessions (user_id -> session)
    sessions: Arc<RwLock<HashMap<i32, WebSocketSession>>>,
    /// Message channels (user_id -> sender)
    channels: Arc<RwLock<HashMap<i32, mpsc::UnboundedSender<SignalingMessage>>>>,
}

impl SignalingServer {
    /// Create new signaling server
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register WebSocket session
    #[instrument(skip(self, tx))]
    pub fn register_session(
        &self,
        user_id: i32,
        session_id: String,
        tx: mpsc::UnboundedSender<SignalingMessage>,
    ) {
        let session = WebSocketSession {
            user_id,
            session_id: session_id.clone(),
            connected_at: Utc::now(),
            last_ping: Utc::now(),
        };

        self.sessions.write().insert(user_id, session);
        self.channels.write().insert(user_id, tx);

        info!(
            user_id = user_id,
            session_id = %session_id,
            "WebSocket session registered"
        );
    }

    /// Unregister WebSocket session
    #[instrument(skip(self))]
    pub fn unregister_session(&self, user_id: i32) {
        self.sessions.write().remove(&user_id);
        self.channels.write().remove(&user_id);

        info!(user_id = user_id, "WebSocket session unregistered");
    }

    /// Send message to peer
    #[instrument(skip(self, message))]
    pub fn send_to_peer(&self, user_id: i32, message: SignalingMessage) -> Result<(), String> {
        let channels = self.channels.read();
        
        if let Some(tx) = channels.get(&user_id) {
            tx.send(message)
                .map_err(|_| "Failed to send message".to_string())?;
            
            debug!(user_id = user_id, "Message sent to peer");
            Ok(())
        } else {
            Err(format!("User {} not connected", user_id))
        }
    }

    /// Handle incoming signaling message
    #[instrument(skip(self, message))]
    pub fn handle_message(&self, message: SignalingMessage) -> Result<(), String> {
        match &message {
            SignalingMessage::Offer { from, to, transfer_id, .. } => {
                info!(
                    from = from,
                    to = to,
                    transfer_id = %transfer_id,
                    "P2P offer received"
                );
                self.send_to_peer(*to, message)?;
            }
            SignalingMessage::Answer { from, to, transfer_id, .. } => {
                info!(
                    from = from,
                    to = to,
                    transfer_id = %transfer_id,
                    "P2P answer received"
                );
                self.send_to_peer(*to, message)?;
            }
            SignalingMessage::IceCandidate { from, to, transfer_id, .. } => {
                debug!(
                    from = from,
                    to = to,
                    transfer_id = %transfer_id,
                    "ICE candidate received"
                );
                self.send_to_peer(*to, message)?;
            }
            SignalingMessage::Connected { transfer_id, peer_id } => {
                info!(
                    transfer_id = %transfer_id,
                    peer_id = peer_id,
                    "P2P connection established"
                );
            }
            SignalingMessage::Failed { transfer_id, reason } => {
                warn!(
                    transfer_id = %transfer_id,
                    reason = %reason,
                    "P2P connection failed"
                );
            }
            SignalingMessage::Ping => {
                // Update last ping time
            }
            SignalingMessage::Pong => {
                // Heartbeat response
            }
        }

        Ok(())
    }

    /// Check if user is online
    pub fn is_online(&self, user_id: i32) -> bool {
        self.sessions.read().contains_key(&user_id)
    }

    /// Get online users count
    pub fn online_count(&self) -> usize {
        self.sessions.read().len()
    }

    /// Get session info
    pub fn get_session(&self, user_id: i32) -> Option<WebSocketSession> {
        self.sessions.read().get(&user_id).cloned()
    }
}

impl Default for SignalingServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_registration() {
        let server = SignalingServer::new();
        let (tx, _rx) = mpsc::unbounded_channel();

        server.register_session(1, "session_123".to_string(), tx);
        
        assert!(server.is_online(1));
        assert_eq!(server.online_count(), 1);
    }

    #[tokio::test]
    async fn test_session_unregistration() {
        let server = SignalingServer::new();
        let (tx, _rx) = mpsc::unbounded_channel();

        server.register_session(1, "session_123".to_string(), tx);
        assert!(server.is_online(1));

        server.unregister_session(1);
        assert!(!server.is_online(1));
    }

    #[tokio::test]
    async fn test_message_routing() {
        let server = SignalingServer::new();
        let (tx1, mut rx1) = mpsc::unbounded_channel();
        let (tx2, _rx2) = mpsc::unbounded_channel();

        server.register_session(1, "session_1".to_string(), tx1);
        server.register_session(2, "session_2".to_string(), tx2);

        let message = SignalingMessage::Offer {
            from: 2,
            to: 1,
            transfer_id: "transfer_123".to_string(),
            sdp: "sdp_data".to_string(),
        };

        server.handle_message(message.clone()).unwrap();

        // User 1 should receive the message
        let received = rx1.recv().await.unwrap();
        match received {
            SignalingMessage::Offer { from, to, .. } => {
                assert_eq!(from, 2);
                assert_eq!(to, 1);
            }
            _ => panic!("Wrong message type"),
        }
    }
}
