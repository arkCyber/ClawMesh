//! Direct messaging between friends
//!
//! Provides one-on-one private messaging functionality for friends

use crate::{MessagePriority, MessageStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Direct message between two users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectMessage {
    /// Message ID
    pub id: i64,
    /// Sender user ID
    pub sender_id: i32,
    /// Recipient user ID
    pub recipient_id: i32,
    /// Message content
    pub content: String,
    /// Message priority
    pub priority: MessagePriority,
    /// Message status
    pub status: MessageStatus,
    /// Reply to message ID
    pub reply_to_id: Option<i64>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Read timestamp
    pub read_at: Option<DateTime<Utc>>,
    /// Is message deleted by sender
    pub deleted_by_sender: bool,
    /// Is message deleted by recipient
    pub deleted_by_recipient: bool,
    /// Attachment URLs
    pub attachments: Vec<String>,
}

impl DirectMessage {
    /// Create a new direct message
    pub fn new(sender_id: i32, recipient_id: i32, content: String) -> Self {
        Self {
            id: 0,
            sender_id,
            recipient_id,
            content,
            priority: MessagePriority::Normal,
            status: MessageStatus::Sent,
            reply_to_id: None,
            created_at: Utc::now(),
            read_at: None,
            deleted_by_sender: false,
            deleted_by_recipient: false,
            attachments: vec![],
        }
    }

    /// Create a reply to another message
    pub fn reply(sender_id: i32, recipient_id: i32, content: String, reply_to: i64) -> Self {
        let mut msg = Self::new(sender_id, recipient_id, content);
        msg.reply_to_id = Some(reply_to);
        msg
    }

    /// Mark message as read
    pub fn mark_read(&mut self) {
        self.status = MessageStatus::Read;
        self.read_at = Some(Utc::now());
    }

    /// Mark message as delivered
    pub fn mark_delivered(&mut self) {
        if self.status == MessageStatus::Sent {
            self.status = MessageStatus::Delivered;
        }
    }

    /// Delete message for sender
    pub fn delete_for_sender(&mut self) {
        self.deleted_by_sender = true;
    }

    /// Delete message for recipient
    pub fn delete_for_recipient(&mut self) {
        self.deleted_by_recipient = true;
    }

    /// Check if message is visible to a user
    pub fn is_visible_to(&self, user_id: i32) -> bool {
        if user_id == self.sender_id {
            !self.deleted_by_sender
        } else if user_id == self.recipient_id {
            !self.deleted_by_recipient
        } else {
            false
        }
    }

    /// Check if message is unread
    pub fn is_unread(&self) -> bool {
        self.status != MessageStatus::Read
    }
}

/// Form for creating a direct message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectMessageForm {
    /// Sender user ID
    pub sender_id: i32,
    /// Recipient user ID
    pub recipient_id: i32,
    /// Message content
    pub content: String,
    /// Message priority
    pub priority: Option<MessagePriority>,
    /// Reply to message ID
    pub reply_to_id: Option<i64>,
    /// Attachment URLs
    pub attachments: Option<Vec<String>>,
}

impl DirectMessageForm {
    /// Create a new direct message form
    pub fn new(sender_id: i32, recipient_id: i32, content: String) -> Self {
        Self {
            sender_id,
            recipient_id,
            content,
            priority: None,
            reply_to_id: None,
            attachments: None,
        }
    }

    /// Set message priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Set reply to message
    pub fn as_reply_to(mut self, message_id: i64) -> Self {
        self.reply_to_id = Some(message_id);
        self
    }

    /// Add attachments
    pub fn with_attachments(mut self, attachments: Vec<String>) -> Self {
        self.attachments = Some(attachments);
        self
    }
}

/// Conversation between two users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Conversation ID (derived from user IDs)
    pub id: String,
    /// First user ID (lower)
    pub user_id_1: i32,
    /// Second user ID (higher)
    pub user_id_2: i32,
    /// Last message preview
    pub last_message: Option<String>,
    /// Last message timestamp
    pub last_message_at: Option<DateTime<Utc>>,
    /// Unread count for user_1
    pub unread_count_1: i32,
    /// Unread count for user_2
    pub unread_count_2: i32,
    /// Is conversation muted by user_1
    pub muted_by_1: bool,
    /// Is conversation muted by user_2
    pub muted_by_2: bool,
}

impl Conversation {
    /// Create a new conversation between two users
    pub fn new(user_id_a: i32, user_id_b: i32) -> Self {
        let (user_id_1, user_id_2) = if user_id_a < user_id_b {
            (user_id_a, user_id_b)
        } else {
            (user_id_b, user_id_a)
        };

        Self {
            id: format!("conv_{}_{}", user_id_1, user_id_2),
            user_id_1,
            user_id_2,
            last_message: None,
            last_message_at: None,
            unread_count_1: 0,
            unread_count_2: 0,
            muted_by_1: false,
            muted_by_2: false,
        }
    }

    /// Get the other user in the conversation
    pub fn get_other_user(&self, user_id: i32) -> Option<i32> {
        if self.user_id_1 == user_id {
            Some(self.user_id_2)
        } else if self.user_id_2 == user_id {
            Some(self.user_id_1)
        } else {
            None
        }
    }

    /// Get unread count for a user
    pub fn get_unread_count(&self, user_id: i32) -> i32 {
        if self.user_id_1 == user_id {
            self.unread_count_1
        } else if self.user_id_2 == user_id {
            self.unread_count_2
        } else {
            0
        }
    }

    /// Check if conversation is muted for a user
    pub fn is_muted_for(&self, user_id: i32) -> bool {
        if self.user_id_1 == user_id {
            self.muted_by_1
        } else if self.user_id_2 == user_id {
            self.muted_by_2
        } else {
            false
        }
    }

    /// Update last message
    pub fn update_last_message(&mut self, content: &str, sender_id: i32) {
        self.last_message = Some(content.chars().take(100).collect());
        self.last_message_at = Some(Utc::now());
        
        // Increment unread count for the other user
        if self.user_id_1 == sender_id {
            self.unread_count_2 += 1;
        } else if self.user_id_2 == sender_id {
            self.unread_count_1 += 1;
        }
    }

    /// Mark all messages as read for a user
    pub fn mark_read_for(&mut self, user_id: i32) {
        if self.user_id_1 == user_id {
            self.unread_count_1 = 0;
        } else if self.user_id_2 == user_id {
            self.unread_count_2 = 0;
        }
    }
}

/// Conversation summary for list display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    /// Conversation ID
    pub id: String,
    /// Other user's ID
    pub other_user_id: i32,
    /// Other user's username
    pub other_username: String,
    /// Other user's display name
    pub other_display_name: Option<String>,
    /// Other user's avatar
    pub other_avatar: Option<String>,
    /// Last message preview
    pub last_message: Option<String>,
    /// Last message timestamp
    pub last_message_at: Option<DateTime<Utc>>,
    /// Unread message count
    pub unread_count: i32,
    /// Is conversation muted
    pub is_muted: bool,
    /// Is other user online
    pub is_online: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_message_creation() {
        let msg = DirectMessage::new(1, 2, "Hello!".to_string());
        assert_eq!(msg.sender_id, 1);
        assert_eq!(msg.recipient_id, 2);
        assert_eq!(msg.content, "Hello!");
        assert!(msg.is_unread());
    }

    #[test]
    fn test_direct_message_reply() {
        let msg = DirectMessage::reply(2, 1, "Hi back!".to_string(), 100);
        assert_eq!(msg.reply_to_id, Some(100));
    }

    #[test]
    fn test_mark_read() {
        let mut msg = DirectMessage::new(1, 2, "Test".to_string());
        assert!(msg.is_unread());
        msg.mark_read();
        assert!(!msg.is_unread());
        assert!(msg.read_at.is_some());
    }

    #[test]
    fn test_visibility() {
        let mut msg = DirectMessage::new(1, 2, "Test".to_string());
        assert!(msg.is_visible_to(1));
        assert!(msg.is_visible_to(2));
        assert!(!msg.is_visible_to(3));
        
        msg.delete_for_sender();
        assert!(!msg.is_visible_to(1));
        assert!(msg.is_visible_to(2));
    }

    #[test]
    fn test_conversation_creation() {
        let conv = Conversation::new(10, 5);
        // Should normalize order
        assert_eq!(conv.user_id_1, 5);
        assert_eq!(conv.user_id_2, 10);
        assert_eq!(conv.id, "conv_5_10");
    }

    #[test]
    fn test_conversation_other_user() {
        let conv = Conversation::new(1, 2);
        assert_eq!(conv.get_other_user(1), Some(2));
        assert_eq!(conv.get_other_user(2), Some(1));
        assert_eq!(conv.get_other_user(3), None);
    }

    #[test]
    fn test_conversation_unread() {
        let mut conv = Conversation::new(1, 2);
        conv.update_last_message("Hello", 1);
        assert_eq!(conv.get_unread_count(1), 0);
        assert_eq!(conv.get_unread_count(2), 1);
        
        conv.mark_read_for(2);
        assert_eq!(conv.get_unread_count(2), 0);
    }

    #[test]
    fn test_message_form_builder() {
        let form = DirectMessageForm::new(1, 2, "Test".to_string())
            .with_priority(MessagePriority::High)
            .as_reply_to(100);
        
        assert_eq!(form.priority, Some(MessagePriority::High));
        assert_eq!(form.reply_to_id, Some(100));
    }
}
