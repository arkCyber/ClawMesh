//! Message Persistence Layer
//!
//! Provides database persistence for offline messages using Lemmy's PrivateMessage system.
//! Integrates with Lemmy's existing database infrastructure for reliability.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};
use anyhow::Result;
use lemmy_db_schema::source::private_message::{PrivateMessage, PrivateMessageInsertForm, PrivateMessageUpdateForm};
use lemmy_diesel_utils::{connection::DbPool, traits::Crud};

use crate::offline_cache::{CachedMessage, MessagePriority};

/// Database schema for offline messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineMessageRecord {
    pub id: i64,
    pub sender_id: i32,
    pub recipient_id: i32,
    pub content: String,
    pub priority: i16,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub delivery_attempts: i32,
    pub last_attempt: Option<DateTime<Utc>>,
    pub delivered: bool,
    pub attachments: Vec<String>,
}

impl From<CachedMessage> for OfflineMessageRecord {
    fn from(msg: CachedMessage) -> Self {
        Self {
            id: msg.id,
            sender_id: msg.sender_id,
            recipient_id: msg.recipient_id,
            content: msg.content,
            priority: msg.priority as i16,
            created_at: msg.created_at,
            expires_at: msg.expires_at,
            delivery_attempts: msg.delivery_attempts as i32,
            last_attempt: msg.last_attempt,
            delivered: msg.delivered,
            attachments: msg.attachments,
        }
    }
}

impl From<OfflineMessageRecord> for CachedMessage {
    fn from(record: OfflineMessageRecord) -> Self {
        Self {
            id: record.id,
            sender_id: record.sender_id,
            recipient_id: record.recipient_id,
            content: record.content,
            priority: match record.priority {
                0 => MessagePriority::Low,
                1 => MessagePriority::Normal,
                2 => MessagePriority::High,
                _ => MessagePriority::Urgent,
            },
            created_at: record.created_at,
            expires_at: record.expires_at,
            delivery_attempts: record.delivery_attempts as u32,
            last_attempt: record.last_attempt,
            delivered: record.delivered,
            attachments: record.attachments,
        }
    }
}

/// Persistence trait for dependency injection
#[async_trait::async_trait]
pub trait MessagePersistenceBackend: Send + Sync {
    async fn save_message(&self, message: &CachedMessage) -> Result<()>;
    async fn load_messages_for_user(&self, user_id: i32, limit: i64) -> Result<Vec<CachedMessage>>;
    async fn mark_delivered(&self, message_id: i64) -> Result<()>;
    async fn update_delivery_attempt(&self, message_id: i64) -> Result<()>;
    async fn delete_expired(&self) -> Result<usize>;
    async fn get_message_count(&self, user_id: i32) -> Result<i64>;
    async fn batch_save(&self, messages: &[CachedMessage]) -> Result<usize>;
}

/// Message persistence service using Lemmy's database
pub struct MessagePersistence {
    pool: DbPool<'static>,
}

impl MessagePersistence {
    /// Create a new persistence service with database pool
    pub fn new(pool: DbPool<'static>) -> Self {
        Self { pool }
    }

    /// Save message to database using Lemmy's PrivateMessage
    #[instrument(skip(self, message))]
    pub async fn save_message(
        &self,
        message: &CachedMessage,
    ) -> Result<()> {
        let form = PrivateMessageInsertForm {
            creator_id: message.sender_id.into(),
            recipient_id: message.recipient_id.into(),
            content: message.content.clone(),
            published: Some(message.created_at),
            ..Default::default()
        };
        
        PrivateMessage::create(&mut self.pool, &form).await?;
        debug!(message_id = message.id, "Message persisted to database");
        Ok(())
    }

    /// Load messages for a user from Lemmy's database
    #[instrument(skip(self))]
    pub async fn load_messages_for_user(
        &self,
        user_id: i32,
        limit: i64,
    ) -> Result<Vec<CachedMessage>> {
        // Query unread private messages for the user
        use lemmy_db_schema::schema::private_message::dsl::*;
        use diesel::prelude::*;
        use lemmy_diesel_utils::connection::get_conn;
        
        let conn = &mut get_conn(&mut self.pool).await?;
        let messages: Vec<PrivateMessage> = private_message
            .filter(recipient_id.eq(user_id))
            .filter(read.eq(false))
            .limit(limit)
            .load(conn)
            .await?;
        
        let cached_messages = messages.into_iter().map(|pm| {
            CachedMessage {
                id: pm.id.0 as i64,
                sender_id: pm.creator_id.0,
                recipient_id: pm.recipient_id.0,
                content: pm.content,
                priority: MessagePriority::Normal,
                created_at: pm.published,
                expires_at: pm.published + chrono::Duration::days(30),
                delivery_attempts: 0,
                last_attempt: None,
                delivered: pm.read,
                attachments: vec![],
            }
        }).collect();
        
        debug!(user_id = user_id, count = cached_messages.len(), "Loaded messages from database");
        Ok(cached_messages)
    }

    /// Mark message as delivered (read) in Lemmy's database
    #[instrument(skip(self))]
    pub async fn mark_delivered(
        &self,
        message_id: i64,
    ) -> Result<()> {
        use lemmy_db_schema::newtypes::PrivateMessageId;
        
        let form = PrivateMessageUpdateForm {
            read: Some(true),
            ..Default::default()
        };
        
        PrivateMessage::update(&mut self.pool, PrivateMessageId(message_id as i32), &form).await?;
        debug!(message_id = message_id, "Message marked as delivered");
        Ok(())
    }

    /// Update delivery attempt (no-op for Lemmy's PrivateMessage)
    #[instrument(skip(self))]
    pub async fn update_delivery_attempt(
        &self,
        message_id: i64,
    ) -> Result<()> {
        // Lemmy's PrivateMessage doesn't track delivery attempts
        // This is handled by the offline cache layer
        debug!(message_id = message_id, "Delivery attempt recorded");
        Ok(())
    }

    /// Delete old read messages from Lemmy's database
    #[instrument(skip(self))]
    pub async fn delete_expired(
        &self,
    ) -> Result<usize> {
        use lemmy_db_schema::schema::private_message::dsl::*;
        use diesel::prelude::*;
        use lemmy_diesel_utils::connection::get_conn;
        
        let conn = &mut get_conn(&mut self.pool).await?;
        let cutoff = Utc::now() - chrono::Duration::days(30);
        
        let deleted = diesel::delete(
            private_message
                .filter(read.eq(true))
                .filter(published.lt(cutoff))
        )
        .execute(conn)
        .await?;
        
        info!(count = deleted, "Deleted expired messages");
        Ok(deleted)
    }

    /// Get unread message count for user from Lemmy's database
    pub async fn get_message_count(
        &self,
        user_id: i32,
    ) -> Result<i64> {
        use lemmy_db_schema::schema::private_message::dsl::*;
        use diesel::prelude::*;
        use lemmy_diesel_utils::connection::get_conn;
        
        let conn = &mut get_conn(&mut self.pool).await?;
        let count = private_message
            .filter(recipient_id.eq(user_id))
            .filter(read.eq(false))
            .count()
            .get_result::<i64>(conn)
            .await?;
        
        debug!(user_id = user_id, count = count, "Got message count");
        Ok(count)
    }

    /// Batch save messages to Lemmy's database
    #[instrument(skip(self, messages))]
    pub async fn batch_save(
        &self,
        messages: &[CachedMessage],
    ) -> Result<usize> {
        use diesel::prelude::*;
        use lemmy_diesel_utils::connection::get_conn;
        use lemmy_db_schema::schema::private_message;
        
        let forms: Vec<PrivateMessageInsertForm> = messages
            .iter()
            .map(|m| PrivateMessageInsertForm {
                creator_id: m.sender_id.into(),
                recipient_id: m.recipient_id.into(),
                content: m.content.clone(),
                published: Some(m.created_at),
                ..Default::default()
            })
            .collect();
        
        let conn = &mut get_conn(&mut self.pool).await?;
        let inserted = diesel::insert_into(private_message::table)
            .values(&forms)
            .execute(conn)
            .await?;
        
        debug!(count = inserted, "Batch saved messages");
        Ok(inserted)
    }
}

// Database schema would be defined here in production
// For now, we use the OfflineMessageRecord struct as the schema definition

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_conversion() {
        let msg = CachedMessage::new(
            1,
            100,
            200,
            "Test".to_string(),
            MessagePriority::High,
        );
        
        let record = OfflineMessageRecord::from(msg.clone());
        assert_eq!(record.sender_id, 100);
        assert_eq!(record.priority, 2); // High = 2
        
        let converted = CachedMessage::from(record);
        assert_eq!(converted.sender_id, msg.sender_id);
        assert_eq!(converted.priority, MessagePriority::High);
    }

    #[test]
    fn test_priority_conversion() {
        let priorities = vec![
            (MessagePriority::Low, 0),
            (MessagePriority::Normal, 1),
            (MessagePriority::High, 2),
            (MessagePriority::Urgent, 3),
        ];
        
        for (priority, expected) in priorities {
            let msg = CachedMessage::new(1, 1, 2, "Test".to_string(), priority);
            let record = OfflineMessageRecord::from(msg);
            assert_eq!(record.priority, expected);
        }
    }
}
