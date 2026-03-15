//! In-app notification storage and management

use crate::{Notification, DeliveryStatus};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// In-app notification
pub type InAppNotification = Notification;

/// Notification storage
pub struct NotificationStore {
    /// User notifications: user_id -> notifications
    notifications: Arc<RwLock<HashMap<i32, Vec<Notification>>>>,
}

impl NotificationStore {
    /// Create a new notification store
    #[must_use]
    pub fn new() -> Self {
        Self {
            notifications: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add notification for user
    ///
    /// # Errors
    /// Returns error if lock is poisoned
    pub fn add(&self, notification: Notification) -> Result<()> {
        let mut store = self.notifications.write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        
        store.entry(notification.recipient_id)
            .or_insert_with(Vec::new)
            .push(notification);
        
        Ok(())
    }

    /// Get notifications for user
    ///
    /// # Errors
    /// Returns error if lock is poisoned
    pub fn get_for_user(&self, user_id: i32) -> Result<Vec<Notification>> {
        let store = self.notifications.read()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        
        Ok(store.get(&user_id).cloned().unwrap_or_default())
    }

    /// Get unread notifications for user
    ///
    /// # Errors
    /// Returns error if lock is poisoned
    pub fn get_unread(&self, user_id: i32) -> Result<Vec<Notification>> {
        let notifications = self.get_for_user(user_id)?;
        Ok(notifications.into_iter()
            .filter(|n| !n.is_read())
            .collect())
    }

    /// Mark notification as read
    ///
    /// # Errors
    /// Returns error if lock is poisoned or notification not found
    pub fn mark_read(&self, user_id: i32, notification_id: &str) -> Result<()> {
        let mut store = self.notifications.write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        
        if let Some(notifications) = store.get_mut(&user_id) {
            if let Some(notification) = notifications.iter_mut()
                .find(|n| n.id == notification_id) {
                notification.mark_read();
                return Ok(());
            }
        }
        
        Err(anyhow::anyhow!("Notification not found"))
    }

    /// Mark all notifications as read for user
    ///
    /// # Errors
    /// Returns error if lock is poisoned
    pub fn mark_all_read(&self, user_id: i32) -> Result<()> {
        let mut store = self.notifications.write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        
        if let Some(notifications) = store.get_mut(&user_id) {
            for notification in notifications {
                notification.mark_read();
            }
        }
        
        Ok(())
    }

    /// Delete notification
    ///
    /// # Errors
    /// Returns error if lock is poisoned
    pub fn delete(&self, user_id: i32, notification_id: &str) -> Result<()> {
        let mut store = self.notifications.write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        
        if let Some(notifications) = store.get_mut(&user_id) {
            notifications.retain(|n| n.id != notification_id);
        }
        
        Ok(())
    }

    /// Clear all notifications for user
    ///
    /// # Errors
    /// Returns error if lock is poisoned
    pub fn clear_all(&self, user_id: i32) -> Result<()> {
        let mut store = self.notifications.write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        
        store.remove(&user_id);
        Ok(())
    }

    /// Get notification count for user
    ///
    /// # Errors
    /// Returns error if lock is poisoned
    pub fn count(&self, user_id: i32) -> Result<usize> {
        let store = self.notifications.read()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        
        Ok(store.get(&user_id).map(|n| n.len()).unwrap_or(0))
    }

    /// Get unread count for user
    ///
    /// # Errors
    /// Returns error if lock is poisoned
    pub fn unread_count(&self, user_id: i32) -> Result<usize> {
        let notifications = self.get_unread(user_id)?;
        Ok(notifications.len())
    }
}

impl Default for NotificationStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NotificationType, NotificationPriority};

    #[test]
    fn test_notification_store() {
        let store = NotificationStore::new();
        
        let notification = Notification::new(
            1,
            NotificationType::NewMessage,
            NotificationPriority::Normal,
            "Test".to_string(),
            "Test message".to_string(),
        );
        
        store.add(notification).expect("Failed to add notification");
        
        let count = store.count(1).expect("Failed to get count");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_get_unread() {
        let store = NotificationStore::new();
        
        let notification = Notification::new(
            1,
            NotificationType::NewMessage,
            NotificationPriority::Normal,
            "Test".to_string(),
            "Test message".to_string(),
        );
        
        store.add(notification).expect("Failed to add notification");
        
        let unread = store.get_unread(1).expect("Failed to get unread");
        assert_eq!(unread.len(), 1);
    }

    #[test]
    fn test_mark_read() {
        let store = NotificationStore::new();
        
        let notification = Notification::new(
            1,
            NotificationType::NewMessage,
            NotificationPriority::Normal,
            "Test".to_string(),
            "Test message".to_string(),
        );
        
        let id = notification.id.clone();
        store.add(notification).expect("Failed to add notification");
        
        store.mark_read(1, &id).expect("Failed to mark read");
        
        let unread = store.get_unread(1).expect("Failed to get unread");
        assert_eq!(unread.len(), 0);
    }

    #[test]
    fn test_mark_all_read() {
        let store = NotificationStore::new();
        
        for i in 0..3 {
            let notification = Notification::new(
                1,
                NotificationType::NewMessage,
                NotificationPriority::Normal,
                format!("Test {}", i),
                "Test message".to_string(),
            );
            store.add(notification).expect("Failed to add notification");
        }
        
        store.mark_all_read(1).expect("Failed to mark all read");
        
        let unread = store.get_unread(1).expect("Failed to get unread");
        assert_eq!(unread.len(), 0);
    }

    #[test]
    fn test_delete() {
        let store = NotificationStore::new();
        
        let notification = Notification::new(
            1,
            NotificationType::NewMessage,
            NotificationPriority::Normal,
            "Test".to_string(),
            "Test message".to_string(),
        );
        
        let id = notification.id.clone();
        store.add(notification).expect("Failed to add notification");
        
        store.delete(1, &id).expect("Failed to delete");
        
        let count = store.count(1).expect("Failed to get count");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_clear_all() {
        let store = NotificationStore::new();
        
        for i in 0..5 {
            let notification = Notification::new(
                1,
                NotificationType::NewMessage,
                NotificationPriority::Normal,
                format!("Test {}", i),
                "Test message".to_string(),
            );
            store.add(notification).expect("Failed to add notification");
        }
        
        store.clear_all(1).expect("Failed to clear all");
        
        let count = store.count(1).expect("Failed to get count");
        assert_eq!(count, 0);
    }
}
