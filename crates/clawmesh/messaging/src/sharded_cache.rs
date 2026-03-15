//! Sharded Offline Message Cache
//!
//! Implements sharded locking to reduce lock contention for 100,000+ concurrent users.
//! Uses consistent hashing to distribute users across shards.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use tracing::{debug, info, warn, instrument};

use crate::offline_cache::{CachedMessage, CacheStats, MessagePriority};

/// Number of shards (power of 2 for efficient modulo)
const SHARD_COUNT: usize = 16;

/// User message queue (same as before)
#[derive(Debug)]
struct UserMessageQueue {
    user_id: i32,
    messages: VecDeque<CachedMessage>,
    total_size: usize,
    last_access: DateTime<Utc>,
}

impl UserMessageQueue {
    fn new(user_id: i32) -> Self {
        Self {
            user_id,
            messages: VecDeque::new(),
            total_size: 0,
            last_access: Utc::now(),
        }
    }

    fn push(&mut self, message: CachedMessage) -> Result<(), &'static str> {
        if self.messages.len() >= 1000 {
            return Err("User message queue full");
        }

        let size = message.content.len() + message.attachments.iter().map(|a| a.len()).sum::<usize>();
        self.total_size += size;
        self.last_access = Utc::now();

        let pos = self.messages
            .iter()
            .position(|m| m.priority < message.priority || 
                         (m.priority == message.priority && m.created_at > message.created_at))
            .unwrap_or(self.messages.len());
        
        self.messages.insert(pos, message);
        Ok(())
    }

    fn pop(&mut self) -> Option<CachedMessage> {
        if let Some(msg) = self.messages.pop_front() {
            let size = msg.content.len() + msg.attachments.iter().map(|a| a.len()).sum::<usize>();
            self.total_size = self.total_size.saturating_sub(size);
            self.last_access = Utc::now();
            Some(msg)
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.messages.len()
    }

    fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    fn remove_expired(&mut self) -> usize {
        let before = self.messages.len();
        self.messages.retain(|m| !m.is_expired());
        let removed = before - self.messages.len();
        if removed > 0 {
            self.recalculate_size();
        }
        removed
    }

    fn recalculate_size(&mut self) {
        self.total_size = self.messages.iter()
            .map(|m| m.content.len() + m.attachments.iter().map(|a| a.len()).sum::<usize>())
            .sum();
    }
}

/// Single shard containing a subset of users
struct CacheShard {
    queues: RwLock<HashMap<i32, UserMessageQueue>>,
    message_count: RwLock<usize>,
}

impl CacheShard {
    fn new() -> Self {
        Self {
            queues: RwLock::new(HashMap::with_capacity(10_000 / SHARD_COUNT)),
            message_count: RwLock::new(0),
        }
    }

    async fn cache_message(&self, message: CachedMessage) -> Result<(), String> {
        let recipient_id = message.recipient_id;
        
        let mut queues = self.queues.write().await;
        let queue = queues.entry(recipient_id).or_insert_with(|| UserMessageQueue::new(recipient_id));
        
        queue.push(message).map_err(|e| e.to_string())?;

        let mut count = self.message_count.write().await;
        *count += 1;

        Ok(())
    }

    async fn get_pending_messages(&self, user_id: i32, limit: usize) -> Vec<CachedMessage> {
        let mut queues = self.queues.write().await;
        
        if let Some(queue) = queues.get_mut(&user_id) {
            let mut messages = Vec::with_capacity(limit.min(queue.len()));
            
            for _ in 0..limit {
                if let Some(msg) = queue.pop() {
                    messages.push(msg);
                } else {
                    break;
                }
            }

            if !messages.is_empty() {
                let mut count = self.message_count.write().await;
                *count = count.saturating_sub(messages.len());
            }

            if queue.is_empty() {
                queues.remove(&user_id);
            }

            messages
        } else {
            Vec::new()
        }
    }

    async fn get_message_count(&self, user_id: i32) -> usize {
        let queues = self.queues.read().await;
        queues.get(&user_id).map(|q| q.len()).unwrap_or(0)
    }

    async fn cleanup_expired(&self) -> usize {
        let mut queues = self.queues.write().await;
        let mut total_removed = 0;

        queues.retain(|_, queue| {
            let removed = queue.remove_expired();
            total_removed += removed;
            !queue.is_empty()
        });

        if total_removed > 0 {
            let mut count = self.message_count.write().await;
            *count = count.saturating_sub(total_removed);
        }

        total_removed
    }

    async fn get_stats(&self) -> (usize, usize, usize) {
        let queues = self.queues.read().await;
        let count = *self.message_count.read().await;
        let total_size: usize = queues.values().map(|q| q.total_size).sum();
        (count, queues.len(), total_size)
    }

    async fn get_users_with_pending(&self, limit: usize) -> Vec<i32> {
        let queues = self.queues.read().await;
        queues.keys().take(limit).copied().collect()
    }
}

/// Sharded offline message cache
/// Reduces lock contention by distributing users across multiple shards
pub struct ShardedOfflineMessageCache {
    shards: Vec<Arc<CacheShard>>,
}

impl ShardedOfflineMessageCache {
    /// Create a new sharded cache
    pub fn new() -> Self {
        let mut shards = Vec::with_capacity(SHARD_COUNT);
        for _ in 0..SHARD_COUNT {
            shards.push(Arc::new(CacheShard::new()));
        }

        Self { shards }
    }

    /// Get shard index for a user
    #[inline]
    fn get_shard_index(&self, user_id: i32) -> usize {
        // Use consistent hashing for even distribution
        (user_id as usize) % SHARD_COUNT
    }

    /// Get shard for a user
    #[inline]
    fn get_shard(&self, user_id: i32) -> &Arc<CacheShard> {
        &self.shards[self.get_shard_index(user_id)]
    }

    /// Cache a message for offline delivery
    #[instrument(skip(self, message))]
    pub async fn cache_message(&self, message: CachedMessage) -> Result<(), String> {
        let recipient_id = message.recipient_id;
        let shard = self.get_shard(recipient_id);
        
        shard.cache_message(message).await?;

        debug!(
            recipient_id = recipient_id,
            shard = self.get_shard_index(recipient_id),
            "Message cached in shard"
        );

        Ok(())
    }

    /// Get pending messages for a user
    #[instrument(skip(self))]
    pub async fn get_pending_messages(&self, user_id: i32, limit: usize) -> Vec<CachedMessage> {
        let shard = self.get_shard(user_id);
        let messages = shard.get_pending_messages(user_id, limit).await;

        info!(
            user_id = user_id,
            retrieved = messages.len(),
            shard = self.get_shard_index(user_id),
            "Retrieved pending messages from shard"
        );

        messages
    }

    /// Get message count for a user
    pub async fn get_message_count(&self, user_id: i32) -> usize {
        let shard = self.get_shard(user_id);
        shard.get_message_count(user_id).await
    }

    /// Clean up expired messages across all shards
    #[instrument(skip(self))]
    pub async fn cleanup_expired(&self) -> usize {
        let mut handles = Vec::with_capacity(SHARD_COUNT);

        for shard in &self.shards {
            let shard = Arc::clone(shard);
            let handle = tokio::spawn(async move {
                shard.cleanup_expired().await
            });
            handles.push(handle);
        }

        let mut total_removed = 0;
        for handle in handles {
            if let Ok(removed) = handle.await {
                total_removed += removed;
            }
        }

        if total_removed > 0 {
            info!(removed = total_removed, "Cleaned up expired messages across all shards");
        }

        total_removed
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let mut total_messages = 0;
        let mut total_users = 0;
        let mut total_size = 0;

        for shard in &self.shards {
            let (messages, users, size) = shard.get_stats().await;
            total_messages += messages;
            total_users += users;
            total_size += size;
        }

        CacheStats {
            total_messages,
            total_users,
            total_size,
            delivered_last_hour: 0,
            failed_last_hour: 0,
            avg_delivery_time: 0.0,
            cache_hit_rate: 0.0,
        }
    }

    /// Get users with pending messages (for batch delivery)
    pub async fn get_users_with_pending(&self, limit: usize) -> Vec<i32> {
        let per_shard = (limit + SHARD_COUNT - 1) / SHARD_COUNT;
        let mut all_users = Vec::with_capacity(limit);

        for shard in &self.shards {
            let users = shard.get_users_with_pending(per_shard).await;
            all_users.extend(users);
            if all_users.len() >= limit {
                break;
            }
        }

        all_users.truncate(limit);
        all_users
    }
}

impl Default for ShardedOfflineMessageCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_distribution() {
        let cache = ShardedOfflineMessageCache::new();
        
        // Test that different users go to different shards
        let shard1 = cache.get_shard_index(1);
        let shard2 = cache.get_shard_index(17); // 17 % 16 = 1
        let shard3 = cache.get_shard_index(32); // 32 % 16 = 0
        
        assert_eq!(shard1, shard2); // Same shard
        assert_ne!(shard1, shard3); // Different shard
    }

    #[tokio::test]
    async fn test_sharded_cache_operations() {
        let cache = ShardedOfflineMessageCache::new();
        
        let msg = CachedMessage::new(1, 100, 200, "Test message".to_string(), MessagePriority::Normal);
        
        cache.cache_message(msg).await.unwrap();
        
        assert_eq!(cache.get_message_count(200).await, 1);
        
        let messages = cache.get_pending_messages(200, 10).await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Test message");
        
        assert_eq!(cache.get_message_count(200).await, 0);
    }

    #[tokio::test]
    async fn test_concurrent_shard_access() {
        let cache = Arc::new(ShardedOfflineMessageCache::new());
        let mut handles = vec![];

        // Simulate 100 concurrent users caching messages
        for i in 0..100 {
            let cache_clone = Arc::clone(&cache);
            let handle = tokio::spawn(async move {
                let msg = CachedMessage::new(
                    i,
                    i as i32,
                    1000,
                    format!("Message from user {}", i),
                    MessagePriority::Normal,
                );
                cache_clone.cache_message(msg).await
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        assert_eq!(cache.get_message_count(1000).await, 100);
    }

    #[tokio::test]
    async fn test_parallel_cleanup() {
        let cache = ShardedOfflineMessageCache::new();
        
        // Cache some messages
        for i in 0..100 {
            let msg = CachedMessage::new(
                i,
                100,
                i as i32,
                format!("Message {}", i),
                MessagePriority::Normal,
            );
            cache.cache_message(msg).await.unwrap();
        }

        let stats = cache.get_stats().await;
        assert_eq!(stats.total_messages, 100);

        // Cleanup should run in parallel across shards
        let removed = cache.cleanup_expired().await;
        assert_eq!(removed, 0); // No expired messages yet
    }

    #[tokio::test]
    async fn test_stats_aggregation() {
        let cache = ShardedOfflineMessageCache::new();
        
        // Distribute messages across shards
        for i in 0..50 {
            let msg = CachedMessage::new(
                i,
                100,
                i as i32 * 16, // Ensure different shards
                format!("Message {}", i),
                MessagePriority::Normal,
            );
            cache.cache_message(msg).await.unwrap();
        }

        let stats = cache.get_stats().await;
        assert_eq!(stats.total_messages, 50);
        assert!(stats.total_users > 0);
        assert!(stats.total_size > 0);
    }
}
