//! ClawMesh Real-time Notification System
//! 
//! Provides push notifications, email notifications, and in-app notifications

pub mod push;
pub mod email;
pub mod inapp;
pub mod dispatcher;

pub use push::{PushNotification, PushProvider};
pub use email::{EmailNotification, EmailProvider};
pub use inapp::{InAppNotification, NotificationStore};
pub use dispatcher::{NotificationDispatcher, NotificationConfig};

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Notification type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NotificationType {
    /// New message notification
    NewMessage,
    /// Mention notification
    Mention,
    /// Reply notification
    Reply,
    /// Group invite
    GroupInvite,
    /// System notification
    System,
    /// Custom notification
    Custom,
}

/// Notification priority
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum NotificationPriority {
    /// Low priority
    Low,
    /// Normal priority
    Normal,
    /// High priority
    High,
    /// Urgent priority
    Urgent,
}

/// Notification delivery status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeliveryStatus {
    /// Pending delivery
    Pending,
    /// Successfully delivered
    Delivered,
    /// Failed to deliver
    Failed,
    /// Read by recipient
    Read,
}

/// Base notification structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Notification ID
    pub id: String,
    /// Recipient user ID
    pub recipient_id: i32,
    /// Notification type
    pub notification_type: NotificationType,
    /// Priority
    pub priority: NotificationPriority,
    /// Title
    pub title: String,
    /// Message body
    pub body: String,
    /// Additional data
    pub data: serde_json::Value,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Delivery status
    pub status: DeliveryStatus,
    /// Read timestamp
    pub read_at: Option<DateTime<Utc>>,
}

impl Notification {
    /// Create a new notification
    #[must_use]
    pub fn new(
        recipient_id: i32,
        notification_type: NotificationType,
        priority: NotificationPriority,
        title: String,
        body: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            recipient_id,
            notification_type,
            priority,
            title,
            body,
            data: serde_json::Value::Null,
            created_at: Utc::now(),
            status: DeliveryStatus::Pending,
            read_at: None,
        }
    }

    /// Mark notification as delivered
    pub fn mark_delivered(&mut self) {
        self.status = DeliveryStatus::Delivered;
    }

    /// Mark notification as read
    pub fn mark_read(&mut self) {
        self.status = DeliveryStatus::Read;
        self.read_at = Some(Utc::now());
    }

    /// Mark notification as failed
    pub fn mark_failed(&mut self) {
        self.status = DeliveryStatus::Failed;
    }

    /// Check if notification is read
    #[must_use]
    pub fn is_read(&self) -> bool {
        self.status == DeliveryStatus::Read
    }

    /// Check if notification is urgent
    #[must_use]
    pub fn is_urgent(&self) -> bool {
        self.priority == NotificationPriority::Urgent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let notification = Notification::new(
            1,
            NotificationType::NewMessage,
            NotificationPriority::Normal,
            "New Message".to_string(),
            "You have a new message".to_string(),
        );

        assert_eq!(notification.recipient_id, 1);
        assert_eq!(notification.notification_type, NotificationType::NewMessage);
        assert_eq!(notification.status, DeliveryStatus::Pending);
        assert!(!notification.is_read());
    }

    #[test]
    fn test_mark_read() {
        let mut notification = Notification::new(
            1,
            NotificationType::Mention,
            NotificationPriority::High,
            "Mention".to_string(),
            "You were mentioned".to_string(),
        );

        notification.mark_read();
        assert!(notification.is_read());
        assert!(notification.read_at.is_some());
    }

    #[test]
    fn test_priority_ordering() {
        assert!(NotificationPriority::Urgent > NotificationPriority::High);
        assert!(NotificationPriority::High > NotificationPriority::Normal);
        assert!(NotificationPriority::Normal > NotificationPriority::Low);
    }

    #[test]
    fn test_is_urgent() {
        let urgent = Notification::new(
            1,
            NotificationType::System,
            NotificationPriority::Urgent,
            "Alert".to_string(),
            "Urgent alert".to_string(),
        );
        assert!(urgent.is_urgent());

        let normal = Notification::new(
            1,
            NotificationType::NewMessage,
            NotificationPriority::Normal,
            "Message".to_string(),
            "Normal message".to_string(),
        );
        assert!(!normal.is_urgent());
    }
}
