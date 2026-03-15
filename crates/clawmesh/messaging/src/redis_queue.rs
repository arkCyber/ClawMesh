//! Redis Message Queue Implementation
//!
//! Production-ready Redis-based message queue with full feature implementation.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{debug, info, warn, instrument};
use redis::{Client, RedisError, cmd};
use std::sync::Arc;

use crate::offline_cache::{CachedMessage, MessagePriority};
use crate::queue::{QueueConfig, QueueMessage, QueueStats};

/// Redis-backed message queue
pub struct RedisMessageQueue {
    config: QueueConfig,
    client: Arc<Client>,
}

impl RedisMessageQueue {
    /// Create a new Redis message queue
    pub fn new(config: QueueConfig) -> Result<Self, RedisError> {
        let client = Client::open(config.redis_url.as_str())?;
        Ok(Self {
            config,
            client: Arc::new(client),
        })
    }

    /// Create from existing client
    pub fn with_client(config: QueueConfig, client: Arc<Client>) -> Self {
        Self { config, client }
    }

    /// Enqueue a message
    #[instrument(skip(self, message))]
    pub async fn enqueue(&self, message: CachedMessage) -> Result<String, String> {
        let queue_msg = QueueMessage::new(message);
        let msg_id = queue_msg.id.clone();

        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| format!("Redis connection error: {}", e))?;

        let queue_key = format!("{}:messages", self.config.queue_prefix);
        let msg_json = serde_json::to_string(&queue_msg)
            .map_err(|e| format!("Serialization error: {}", e))?;

        // Push to queue
        cmd("LPUSH")
            .arg(&queue_key)
            .arg(&msg_json)
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| format!("Redis LPUSH error: {}", e))?;

        // Set TTL on message
        let msg_key = format!("{}:msg:{}", self.config.queue_prefix, msg_id);
        cmd("SETEX")
            .arg(&msg_key)
            .arg(self.config.message_ttl)
            .arg(&msg_json)
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| format!("Redis SET error: {}", e))?;

        debug!(
            message_id = %msg_id,
            recipient_id = queue_msg.payload.recipient_id,
            "Message enqueued to Redis"
        );

        Ok(msg_id)
    }

    /// Dequeue a message (blocking)
    #[instrument(skip(self))]
    pub async fn dequeue(&self, timeout: u64) -> Result<Option<QueueMessage>, String> {
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| format!("Redis connection error: {}", e))?;

        let queue_key = format!("{}:messages", self.config.queue_prefix);
        
        // BRPOP with timeout
        let result: Option<(String, String)> = cmd("BRPOP")
            .arg(&queue_key)
            .arg(timeout)
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("Redis BRPOP error: {}", e))?;

        if let Some((_, msg_json)) = result {
            let queue_msg: QueueMessage = serde_json::from_str(&msg_json)
                .map_err(|e| format!("Deserialization error: {}", e))?;
            
            debug!(msg_id = %queue_msg.id, "Message dequeued from Redis");
            Ok(Some(queue_msg))
        } else {
            Ok(None)
        }
    }

    /// Acknowledge message processing
    #[instrument(skip(self))]
    pub async fn ack(&self, message_id: &str) -> Result<(), String> {
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| format!("Redis connection error: {}", e))?;

        let msg_key = format!("{}:msg:{}", self.config.queue_prefix, message_id);
        let stats_key = format!("{}:stats", self.config.queue_prefix);
        
        cmd("DEL")
            .arg(&msg_key)
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| format!("Redis DEL error: {}", e))?;

        // Increment processed counter
        cmd("HINCRBY")
            .arg(&stats_key)
            .arg("processed")
            .arg(1)
            .query_async::<_, i64>(&mut conn)
            .await
            .map_err(|e| format!("Redis HINCRBY error: {}", e))?;
        
        debug!(message_id = message_id, "Message acknowledged");
        Ok(())
    }

    /// Negative acknowledge (requeue for retry)
    #[instrument(skip(self, message))]
    pub async fn nack(&self, mut message: QueueMessage) -> Result<(), String> {
        if message.should_retry(self.config.max_retries) {
            message.retry_count += 1;
            message.next_retry = Some(message.calculate_next_retry());

            let mut conn = self.client.get_multiplexed_async_connection()
                .await
                .map_err(|e| format!("Redis connection error: {}", e))?;

            let retry_queue_key = format!("{}:retry", self.config.queue_prefix);
            let msg_json = serde_json::to_string(&message)
                .map_err(|e| format!("Serialization error: {}", e))?;
            
            let score = message.next_retry.unwrap().timestamp() as f64;
            
            cmd("ZADD")
                .arg(&retry_queue_key)
                .arg(score)
                .arg(&msg_json)
                .query_async::<_, ()>(&mut conn)
                .await
                .map_err(|e| format!("Redis ZADD error: {}", e))?;
            
            info!(
                message_id = %message.id,
                retry_count = message.retry_count,
                next_retry = ?message.next_retry,
                "Message requeued for retry"
            );
        } else {
            self.move_to_dlq(message).await?;
        }

        Ok(())
    }

    /// Move to dead letter queue
    #[instrument(skip(self, message))]
    pub async fn move_to_dlq(&self, message: QueueMessage) -> Result<(), String> {
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| format!("Redis connection error: {}", e))?;

        let dlq_key = format!("{}:dlq", self.config.queue_prefix);
        let stats_key = format!("{}:stats", self.config.queue_prefix);
        let msg_json = serde_json::to_string(&message)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        cmd("LPUSH")
            .arg(&dlq_key)
            .arg(&msg_json)
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| format!("Redis LPUSH error: {}", e))?;

        // Increment failed counter
        cmd("HINCRBY")
            .arg(&stats_key)
            .arg("failed")
            .arg(1)
            .query_async::<_, i64>(&mut conn)
            .await
            .map_err(|e| format!("Redis HINCRBY error: {}", e))?;
        
        warn!(
            message_id = %message.id,
            retry_count = message.retry_count,
            "Message moved to DLQ"
        );
        Ok(())
    }

    /// Process retry queue (move ready messages back to main queue)
    #[instrument(skip(self))]
    pub async fn process_retry_queue(&self) -> Result<usize, String> {
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| format!("Redis connection error: {}", e))?;

        let retry_queue_key = format!("{}:retry", self.config.queue_prefix);
        let queue_key = format!("{}:messages", self.config.queue_prefix);
        let now = Utc::now().timestamp() as f64;

        // Get messages ready for retry
        let messages: Vec<String> = cmd("ZRANGEBYSCORE")
            .arg(&retry_queue_key)
            .arg(0.0)
            .arg(now)
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("Redis ZRANGEBYSCORE error: {}", e))?;

        let count = messages.len();
        
        for msg_json in messages {
            // Move to main queue
            cmd("LPUSH")
                .arg(&queue_key)
                .arg(&msg_json)
                .query_async::<_, ()>(&mut conn)
                .await
                .map_err(|e| format!("Redis LPUSH error: {}", e))?;
            
            // Remove from retry queue
            cmd("ZREM")
                .arg(&retry_queue_key)
                .arg(&msg_json)
                .query_async::<_, ()>(&mut conn)
                .await
                .map_err(|e| format!("Redis ZREM error: {}", e))?;
        }

        if count > 0 {
            info!(count = count, "Processed retry queue");
        }

        Ok(count)
    }

    /// Get queue statistics
    pub async fn get_stats(&self) -> Result<QueueStats, String> {
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| format!("Redis connection error: {}", e))?;

        let queue_key = format!("{}:messages", self.config.queue_prefix);
        let retry_key = format!("{}:retry", self.config.queue_prefix);
        let dlq_key = format!("{}:dlq", self.config.queue_prefix);
        let processing_key = format!("{}:processing", self.config.queue_prefix);

        let pending: i64 = cmd("LLEN").arg(&queue_key).query_async(&mut conn).await.unwrap_or(0);
        let retry: i64 = cmd("ZCARD").arg(&retry_key).query_async(&mut conn).await.unwrap_or(0);
        let dlq: i64 = cmd("LLEN").arg(&dlq_key).query_async(&mut conn).await.unwrap_or(0);
        let processing: i64 = cmd("LLEN").arg(&processing_key).query_async(&mut conn).await.unwrap_or(0);

        Ok(QueueStats {
            pending_messages: pending as usize,
            processing_messages: processing as usize,
            retry_messages: retry as usize,
            dead_letter_messages: dlq as usize,
        })
    }

    /// Clear all queues (for testing)
    #[cfg(test)]
    pub async fn clear_all(&self) -> Result<(), String> {
        let mut conn = self.client.get_multiplexed_async_connection()
            .await
            .map_err(|e| format!("Redis connection error: {}", e))?;

        let keys = vec![
            format!("{}:messages", self.config.queue_prefix),
            format!("{}:retry", self.config.queue_prefix),
            format!("{}:dlq", self.config.queue_prefix),
            format!("{}:processing", self.config.queue_prefix),
            format!("{}:stats", self.config.queue_prefix),
        ];

        for key in keys {
            let _: () = conn.del(&key).await.unwrap_or(());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> QueueConfig {
        QueueConfig {
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            queue_prefix: "clawmesh:test".to_string(),
            message_ttl: 3600,
            max_retries: 3,
            visibility_timeout: 60,
        }
    }

    #[tokio::test]
    #[ignore] // Requires Redis server
    async fn test_enqueue_dequeue() {
        let config = test_config();
        let queue = RedisMessageQueue::new(config).unwrap();
        queue.clear_all().await.unwrap();

        let msg = CachedMessage::new(1, 100, 200, "Test".to_string(), MessagePriority::Normal);
        
        // Enqueue
        let msg_id = queue.enqueue(msg).await.unwrap();
        assert!(!msg_id.is_empty());

        // Dequeue
        let dequeued = queue.dequeue(1).await.unwrap();
        assert!(dequeued.is_some());
        
        let dequeued_msg = dequeued.unwrap();
        assert_eq!(dequeued_msg.id, msg_id);
    }

    #[tokio::test]
    #[ignore] // Requires Redis server
    async fn test_retry_logic() {
        let config = test_config();
        let queue = RedisMessageQueue::new(config).unwrap();
        queue.clear_all().await.unwrap();

        let msg = CachedMessage::new(1, 100, 200, "Test".to_string(), MessagePriority::Normal);
        let queue_msg = QueueMessage::new(msg);

        // NACK should requeue
        queue.nack(queue_msg).await.unwrap();

        let stats = queue.get_stats().await.unwrap();
        assert_eq!(stats.retry_messages, 1);
    }

    #[tokio::test]
    #[ignore] // Requires Redis server
    async fn test_dead_letter_queue() {
        let config = test_config();
        let queue = RedisMessageQueue::new(config.clone()).unwrap();
        queue.clear_all().await.unwrap();

        let msg = CachedMessage::new(1, 100, 200, "Test".to_string(), MessagePriority::Normal);
        let mut queue_msg = QueueMessage::new(msg);
        queue_msg.retry_count = config.max_retries;

        // NACK should move to DLQ
        queue.nack(queue_msg).await.unwrap();

        let stats = queue.get_stats().await.unwrap();
        assert_eq!(stats.dead_letter_messages, 1);
    }
}
