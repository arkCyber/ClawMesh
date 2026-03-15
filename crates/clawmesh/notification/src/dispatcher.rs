//! Notification dispatcher for routing notifications to appropriate channels

use crate::{
    Notification, NotificationPriority, NotificationType,
    PushNotification, PushProvider,
    EmailNotification, EmailProvider,
    NotificationStore,
};
use anyhow::Result;
use std::sync::Arc;

/// Notification configuration
#[derive(Debug, Clone)]
pub struct NotificationConfig {
    /// Enable push notifications
    pub enable_push: bool,
    /// Enable email notifications
    pub enable_email: bool,
    /// Enable in-app notifications
    pub enable_inapp: bool,
    /// Send email for urgent notifications
    pub email_urgent_only: bool,
    /// Batch size for bulk sending
    pub batch_size: usize,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enable_push: true,
            enable_email: true,
            enable_inapp: true,
            email_urgent_only: false,
            batch_size: 100,
        }
    }
}

/// Notification dispatcher
pub struct NotificationDispatcher {
    config: NotificationConfig,
    store: Arc<NotificationStore>,
}

impl NotificationDispatcher {
    /// Create a new notification dispatcher
    #[must_use]
    pub fn new(config: NotificationConfig) -> Self {
        Self {
            config,
            store: Arc::new(NotificationStore::new()),
        }
    }

    /// Dispatch a notification
    ///
    /// # Errors
    /// Returns error if dispatch fails
    pub async fn dispatch(&self, notification: Notification) -> Result<()> {
        // Store in-app notification
        if self.config.enable_inapp {
            self.store.add(notification.clone())?;
        }

        // Determine which channels to use
        let should_push = self.config.enable_push;
        let should_email = self.config.enable_email && 
            (!self.config.email_urgent_only || notification.is_urgent());

        // TODO: Dispatch to push provider
        if should_push {
            tracing::info!("Would send push notification to user {}", notification.recipient_id);
        }

        // TODO: Dispatch to email provider
        if should_email {
            tracing::info!("Would send email notification to user {}", notification.recipient_id);
        }

        Ok(())
    }

    /// Dispatch batch notifications
    ///
    /// # Errors
    /// Returns error if dispatch fails
    pub async fn dispatch_batch(&self, notifications: Vec<Notification>) -> Result<()> {
        for chunk in notifications.chunks(self.config.batch_size) {
            for notification in chunk {
                self.dispatch(notification.clone()).await?;
            }
        }
        Ok(())
    }

    /// Get notification store
    #[must_use]
    pub fn store(&self) -> Arc<NotificationStore> {
        Arc::clone(&self.store)
    }

    /// Get unread count for user
    ///
    /// # Errors
    /// Returns error if operation fails
    pub fn get_unread_count(&self, user_id: i32) -> Result<usize> {
        self.store.unread_count(user_id)
    }

    /// Mark notification as read
    ///
    /// # Errors
    /// Returns error if operation fails
    pub fn mark_read(&self, user_id: i32, notification_id: &str) -> Result<()> {
        self.store.mark_read(user_id, notification_id)
    }

    /// Get notifications for user
    ///
    /// # Errors
    /// Returns error if operation fails
    pub fn get_notifications(&self, user_id: i32) -> Result<Vec<Notification>> {
        self.store.get_for_user(user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dispatcher_creation() {
        let config = NotificationConfig::default();
        let dispatcher = NotificationDispatcher::new(config);
        
        let count = dispatcher.get_unread_count(1).expect("Failed to get count");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_dispatch_notification() {
        let config = NotificationConfig::default();
        let dispatcher = NotificationDispatcher::new(config);
        
        let notification = Notification::new(
            1,
            NotificationType::NewMessage,
            NotificationPriority::Normal,
            "Test".to_string(),
            "Test message".to_string(),
        );
        
        dispatcher.dispatch(notification).await.expect("Failed to dispatch");
        
        let count = dispatcher.get_unread_count(1).expect("Failed to get count");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_dispatch_batch() {
        let config = NotificationConfig::default();
        let dispatcher = NotificationDispatcher::new(config);
        
        let notifications: Vec<Notification> = (0..5)
            .map(|i| Notification::new(
                1,
                NotificationType::NewMessage,
                NotificationPriority::Normal,
                format!("Test {}", i),
                "Test message".to_string(),
            ))
            .collect();
        
        dispatcher.dispatch_batch(notifications).await.expect("Failed to dispatch batch");
        
        let count = dispatcher.get_unread_count(1).expect("Failed to get count");
        assert_eq!(count, 5);
    }

    #[tokio::test]
    async fn test_mark_read() {
        let config = NotificationConfig::default();
        let dispatcher = NotificationDispatcher::new(config);
        
        let notification = Notification::new(
            1,
            NotificationType::NewMessage,
            NotificationPriority::Normal,
            "Test".to_string(),
            "Test message".to_string(),
        );
        
        let id = notification.id.clone();
        dispatcher.dispatch(notification).await.expect("Failed to dispatch");
        
        dispatcher.mark_read(1, &id).expect("Failed to mark read");
        
        let count = dispatcher.get_unread_count(1).expect("Failed to get count");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_config_defaults() {
        let config = NotificationConfig::default();
        assert!(config.enable_push);
        assert!(config.enable_email);
        assert!(config.enable_inapp);
        assert!(!config.email_urgent_only);
        assert_eq!(config.batch_size, 100);
    }
}
