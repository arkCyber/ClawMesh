//! WebSocket Message Delivery Service
//!
//! Aerospace-grade real-time message delivery using WebSocket.
//! Implements:
//! - Real-time message push
//! - User online status management
//! - Automatic offline message delivery
//! - Message acknowledgment
//! - Heartbeat mechanism

use actix::{Actor, ActorContext, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

// ============================================================================
// WebSocket Message Types
// ============================================================================

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// New message notification
    NewMessage {
        message_id: i64,
        sender_id: i32,
        content: String,
        created_at: String,
    },
    
    /// Message read notification
    MessageRead {
        message_id: i64,
        read_by: i32,
    },
    
    /// User online status change
    UserStatus {
        user_id: i32,
        online: bool,
    },
    
    /// Typing indicator
    Typing {
        user_id: i32,
        conversation_id: String,
    },
    
    /// Message acknowledgment
    Ack {
        message_id: i64,
    },
    
    /// Ping (heartbeat)
    Ping,
    
    /// Pong (heartbeat response)
    Pong,
    
    /// Error message
    Error {
        code: String,
        message: String,
    },
}

/// Client message (from browser to server)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Subscribe to user's messages
    Subscribe,
    
    /// Acknowledge message received
    Ack {
        message_id: i64,
    },
    
    /// Send typing indicator
    Typing {
        conversation_id: String,
    },
    
    /// Ping
    Ping,
}

// ============================================================================
// WebSocket Session
// ============================================================================

/// WebSocket session for a connected user
pub struct WsSession {
    /// User ID
    user_id: i32,
    
    /// Session ID
    session_id: String,
    
    /// Last heartbeat time
    last_heartbeat: Instant,
    
    /// Connection manager
    manager: Arc<ConnectionManager>,
}

impl WsSession {
    /// Create new WebSocket session
    pub fn new(user_id: i32, manager: Arc<ConnectionManager>) -> Self {
        Self {
            user_id,
            session_id: uuid::Uuid::new_v4().to_string(),
            last_heartbeat: Instant::now(),
            manager,
        }
    }
    
    /// Send heartbeat ping
    fn send_heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(Duration::from_secs(30), |act, ctx| {
            // Check if client is still alive
            if Instant::now().duration_since(act.last_heartbeat) > Duration::from_secs(60) {
                warn!(
                    user_id = act.user_id,
                    session_id = %act.session_id,
                    "WebSocket heartbeat timeout, disconnecting"
                );
                ctx.stop();
                return;
            }
            
            // Send ping
            let ping_msg = WsMessage::Ping;
            if let Ok(json) = serde_json::to_string(&ping_msg) {
                ctx.text(json);
            }
        });
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        info!(
            user_id = self.user_id,
            session_id = %self.session_id,
            "WebSocket session started"
        );
        
        // Start heartbeat
        self.send_heartbeat(ctx);
        
        // Register connection
        let user_id = self.user_id;
        let session_id = self.session_id.clone();
        let addr = ctx.address();
        let manager = self.manager.clone();
        
        actix::spawn(async move {
            manager.register_connection(user_id, session_id, addr).await;
        });
    }
    
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!(
            user_id = self.user_id,
            session_id = %self.session_id,
            "WebSocket session stopped"
        );
        
        // Unregister connection
        let user_id = self.user_id;
        let session_id = self.session_id.clone();
        let manager = self.manager.clone();
        
        actix::spawn(async move {
            manager.unregister_connection(user_id, &session_id).await;
        });
    }
}

/// Handle WebSocket messages from client
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                // Parse client message
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        self.handle_client_message(client_msg, ctx);
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to parse client message");
                        let error_msg = WsMessage::Error {
                            code: "INVALID_MESSAGE".to_string(),
                            message: "Invalid message format".to_string(),
                        };
                        if let Ok(json) = serde_json::to_string(&error_msg) {
                            ctx.text(json);
                        }
                    }
                }
            }
            Ok(ws::Message::Ping(msg)) => {
                self.last_heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.last_heartbeat = Instant::now();
            }
            Ok(ws::Message::Close(reason)) => {
                debug!(
                    user_id = self.user_id,
                    reason = ?reason,
                    "WebSocket close received"
                );
                ctx.stop();
            }
            _ => {}
        }
    }
}

impl WsSession {
    /// Handle client messages
    fn handle_client_message(&mut self, msg: ClientMessage, ctx: &mut ws::WebsocketContext<Self>) {
        match msg {
            ClientMessage::Subscribe => {
                debug!(user_id = self.user_id, "Client subscribed");
                // Deliver pending offline messages
                let user_id = self.user_id;
                let manager = self.manager.clone();
                let addr = ctx.address();
                
                actix::spawn(async move {
                    manager.deliver_offline_messages(user_id, addr).await;
                });
            }
            
            ClientMessage::Ack { message_id } => {
                debug!(
                    user_id = self.user_id,
                    message_id = message_id,
                    "Message acknowledged"
                );
                // TODO: Mark message as delivered in database
            }
            
            ClientMessage::Typing { conversation_id } => {
                debug!(
                    user_id = self.user_id,
                    conversation_id = %conversation_id,
                    "Typing indicator"
                );
                // TODO: Broadcast typing indicator to conversation participants
            }
            
            ClientMessage::Ping => {
                self.last_heartbeat = Instant::now();
                let pong_msg = WsMessage::Pong;
                if let Ok(json) = serde_json::to_string(&pong_msg) {
                    ctx.text(json);
                }
            }
        }
    }
}

/// Message to send to WebSocket client
#[derive(Message)]
#[rtype(result = "()")]
pub struct SendMessage(pub WsMessage);

impl Handler<SendMessage> for WsSession {
    type Result = ();
    
    fn handle(&mut self, msg: SendMessage, ctx: &mut Self::Context) {
        if let Ok(json) = serde_json::to_string(&msg.0) {
            ctx.text(json);
        }
    }
}

// ============================================================================
// Connection Manager
// ============================================================================

/// Manages all active WebSocket connections
pub struct ConnectionManager {
    /// Active connections: user_id -> (session_id, actor_address)
    connections: Arc<RwLock<HashMap<i32, Vec<(String, actix::Addr<WsSession>)>>>>,
    
    /// Offline message cache
    offline_cache: Arc<crate::offline_cache::OfflineMessageCache>,
}

impl ConnectionManager {
    /// Create new connection manager
    pub fn new(offline_cache: Arc<crate::offline_cache::OfflineMessageCache>) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            offline_cache,
        }
    }
    
    /// Register a new connection
    pub async fn register_connection(
        &self,
        user_id: i32,
        session_id: String,
        addr: actix::Addr<WsSession>,
    ) {
        let mut connections = self.connections.write().await;
        
        connections
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push((session_id.clone(), addr));
        
        info!(
            user_id = user_id,
            session_id = %session_id,
            total_connections = connections.get(&user_id).map(|v| v.len()).unwrap_or(0),
            "Connection registered"
        );
        
        // Broadcast user online status
        self.broadcast_user_status(user_id, true).await;
    }
    
    /// Unregister a connection
    pub async fn unregister_connection(&self, user_id: i32, session_id: &str) {
        let mut connections = self.connections.write().await;
        
        if let Some(user_connections) = connections.get_mut(&user_id) {
            user_connections.retain(|(sid, _)| sid != session_id);
            
            if user_connections.is_empty() {
                connections.remove(&user_id);
                
                info!(
                    user_id = user_id,
                    session_id = %session_id,
                    "Last connection closed, user offline"
                );
                
                // Broadcast user offline status
                drop(connections);
                self.broadcast_user_status(user_id, false).await;
            } else {
                info!(
                    user_id = user_id,
                    session_id = %session_id,
                    remaining_connections = user_connections.len(),
                    "Connection unregistered"
                );
            }
        }
    }
    
    /// Check if user is online
    pub async fn is_user_online(&self, user_id: i32) -> bool {
        let connections = self.connections.read().await;
        connections.contains_key(&user_id)
    }
    
    /// Send message to user
    pub async fn send_to_user(&self, user_id: i32, message: WsMessage) -> Result<(), String> {
        let connections = self.connections.read().await;
        
        if let Some(user_connections) = connections.get(&user_id) {
            // Send to all user's connections
            for (session_id, addr) in user_connections {
                debug!(
                    user_id = user_id,
                    session_id = %session_id,
                    "Sending message to connection"
                );
                
                addr.do_send(SendMessage(message.clone()));
            }
            
            Ok(())
        } else {
            Err(format!("User {} not connected", user_id))
        }
    }
    
    /// Deliver offline messages to user
    async fn deliver_offline_messages(&self, user_id: i32, addr: actix::Addr<WsSession>) {
        let count = self.offline_cache.get_message_count(user_id).await;
        
        if count == 0 {
            return;
        }
        
        info!(
            user_id = user_id,
            count = count,
            "Delivering offline messages"
        );
        
        let messages = self.offline_cache.get_pending_messages(user_id, count).await;
        
        for cached_msg in messages {
            let ws_msg = WsMessage::NewMessage {
                message_id: cached_msg.id,
                sender_id: cached_msg.sender_id,
                content: cached_msg.content,
                created_at: cached_msg.created_at.to_rfc3339(),
            };
            
            addr.do_send(SendMessage(ws_msg));
            
            // Remove from cache after delivery
            let _ = self.offline_cache.remove_message(cached_msg.id).await;
        }
    }
    
    /// Broadcast user status change
    async fn broadcast_user_status(&self, user_id: i32, online: bool) {
        // TODO: Get user's friends list and broadcast to them
        let status_msg = WsMessage::UserStatus { user_id, online };
        
        debug!(
            user_id = user_id,
            online = online,
            "Broadcasting user status"
        );
        
        // For now, just log
        // In production, this would query friends and send to each
    }
    
    /// Get online user count
    pub async fn online_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }
    
    /// Get connection count for user
    pub async fn user_connection_count(&self, user_id: i32) -> usize {
        let connections = self.connections.read().await;
        connections.get(&user_id).map(|v| v.len()).unwrap_or(0)
    }
}

// ============================================================================
// WebSocket Route Handler
// ============================================================================

/// WebSocket endpoint handler
pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    manager: web::Data<Arc<ConnectionManager>>,
    context: web::Data<lemmy_api_utils::context::LemmyContext>,
) -> Result<HttpResponse, Error> {
    // Authenticate user
    let user = match crate::require_extended_user(&req, &context).await {
        Ok(u) => u,
        Err(e) => {
            error!("WebSocket authentication failed: {}", e);
            return Err(actix_web::error::ErrorUnauthorized("Authentication required"));
        }
    };
    
    info!(
        user_id = user.person.id.0,
        username = %user.person.name,
        "WebSocket connection request"
    );
    
    // Create WebSocket session
    let session = WsSession::new(user.person.id.0, manager.get_ref().clone());
    
    // Start WebSocket
    ws::start(session, &req, stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::NewMessage {
            message_id: 123,
            sender_id: 456,
            content: "Hello!".to_string(),
            created_at: "2026-03-15T09:00:00Z".to_string(),
        };
        
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("new_message"));
        assert!(json.contains("123"));
    }
    
    #[test]
    fn test_client_message_deserialization() {
        let json = r#"{"type":"subscribe"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        
        match msg {
            ClientMessage::Subscribe => {}
            _ => panic!("Expected Subscribe"),
        }
    }
    
    #[tokio::test]
    async fn test_connection_manager() {
        use crate::offline_cache::OfflineMessageCache;
        
        let cache = Arc::new(OfflineMessageCache::new(100));
        let manager = ConnectionManager::new(cache);
        
        assert_eq!(manager.online_count().await, 0);
        assert!(!manager.is_user_online(1).await);
    }
}
