//! Real-time message types and serialization

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Message type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// Text message
    Text,
    /// System notification
    Notification,
    /// User presence update
    Presence,
    /// Typing indicator
    Typing,
    /// Heartbeat ping
    Ping,
    /// Heartbeat pong
    Pong,
    /// Join room
    Join,
    /// Leave room
    Leave,
    /// Error message
    Error,
}

/// Client-to-server message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMessage {
    /// Message type
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    /// Target room ID (optional)
    pub room_id: Option<String>,
    /// Message content
    pub content: Option<String>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Server-to-client message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMessage {
    /// Message type
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    /// Sender user ID (optional)
    pub sender_id: Option<i32>,
    /// Target room ID (optional)
    pub room_id: Option<String>,
    /// Message content
    pub content: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl ServerMessage {
    /// Create a new text message
    #[must_use]
    pub fn text(sender_id: i32, room_id: String, content: String) -> Self {
        Self {
            msg_type: MessageType::Text,
            sender_id: Some(sender_id),
            room_id: Some(room_id),
            content: Some(content),
            timestamp: Utc::now(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Create a notification message
    #[must_use]
    pub fn notification(content: String) -> Self {
        Self {
            msg_type: MessageType::Notification,
            sender_id: None,
            room_id: None,
            content: Some(content),
            timestamp: Utc::now(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Create a presence update message
    #[must_use]
    pub fn presence(user_id: i32, status: &str) -> Self {
        Self {
            msg_type: MessageType::Presence,
            sender_id: Some(user_id),
            room_id: None,
            content: Some(status.to_string()),
            timestamp: Utc::now(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Create a pong message
    #[must_use]
    pub fn pong() -> Self {
        Self {
            msg_type: MessageType::Pong,
            sender_id: None,
            room_id: None,
            content: None,
            timestamp: Utc::now(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Create an error message
    #[must_use]
    pub fn error(message: String) -> Self {
        Self {
            msg_type: MessageType::Error,
            sender_id: None,
            room_id: None,
            content: Some(message),
            timestamp: Utc::now(),
            metadata: serde_json::Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = ServerMessage::text(1, "room1".to_string(), "Hello".to_string());
        let json = serde_json::to_string(&msg).expect("Failed to serialize");
        assert!(json.contains("text"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_client_message_deserialization() {
        let json = r#"{"type":"text","room_id":"room1","content":"Hello"}"#;
        let msg: ClientMessage = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(msg.msg_type, MessageType::Text);
        assert_eq!(msg.content, Some("Hello".to_string()));
    }

    #[test]
    fn test_notification_message() {
        let msg = ServerMessage::notification("Test notification".to_string());
        assert_eq!(msg.msg_type, MessageType::Notification);
        assert_eq!(msg.content, Some("Test notification".to_string()));
    }
}
