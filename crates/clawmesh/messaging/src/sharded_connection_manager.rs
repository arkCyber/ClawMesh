//! Sharded Connection Manager for High Concurrency
//!
//! Aerospace-grade connection manager supporting 100,000+ concurrent WebSocket connections.
//! Uses sharding to reduce lock contention and improve scalability.

use actix::Addr;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, info, warn};

use crate::websocket::{WsSession, WsMessage};
use crate::offline_cache::OfflineMessageCache;

/// Number of shards for connection storage
const DEFAULT_SHARD_COUNT: usize = 256;

/// Sharded connection manager for high concurrency
pub struct ShardedConnectionManager {
    /// Connection shards
    shards: Vec<Arc<RwLock<HashMap<i32, Vec<(String, Addr<WsSession>)>>>>>,
    /// Number of shards
    shard_count: usize,
    /// Offline message cache
    offline_cache: Arc<OfflineMessageCache>,
    /// Connection limit per shard
    max_connections_per_shard: usize,
    /// Total connection count (atomic)
    total_connections: Arc<parking_lot::RwLock<usize>>,
}

impl ShardedConnectionManager {
    /// Create new sharded connection manager
    pub fn new(
        offline_cache: Arc<OfflineMessageCache>,
        shard_count: usize,
        max_total_connections: usize,
    ) -> Self {
        let mut shards = Vec::with_capacity(shard_count);
        for _ in 0..shard_count {
            shards.push(Arc::new(RwLock::new(HashMap::new())));
        }
        
        let max_connections_per_shard = max_total_connections / shard_count;
        
        info!(
            shard_count = shard_count,
            max_connections_per_shard = max_connections_per_shard,
            max_total_connections = max_total_connections,
            "Sharded connection manager initialized"
        );
        
        Self {
            shards,
            shard_count,
            offline_cache,
            max_connections_per_shard,
            total_connections: Arc::new(parking_lot::RwLock::new(0)),
        }
    }
    
    /// Create with default configuration (256 shards, 150K max connections)
    pub fn with_defaults(offline_cache: Arc<OfflineMessageCache>) -> Self {
        Self::new(offline_cache, DEFAULT_SHARD_COUNT, 150_000)
    }
    
    /// Get shard index for user ID
    #[inline]
    fn get_shard_index(&self, user_id: i32) -> usize {
        // Use modulo for even distribution
        (user_id as usize) % self.shard_count
    }
    
    /// Get shard for user ID
    #[inline]
    fn get_shard(&self, user_id: i32) -> &Arc<RwLock<HashMap<i32, Vec<(String, Addr<WsSession>)>>>> {
        let index = self.get_shard_index(user_id);
        &self.shards[index]
    }
    
    /// Register a new connection
    pub async fn register_connection(
        &self,
        user_id: i32,
        session_id: String,
        addr: Addr<WsSession>,
    ) -> Result<(), String> {
        // Check total connection limit
        {
            let total = *self.total_connections.read();
            if total >= self.max_connections_per_shard * self.shard_count {
                warn!(
                    total_connections = total,
                    max_connections = self.max_connections_per_shard * self.shard_count,
                    "Connection limit reached"
                );
                return Err("Connection limit reached".to_string());
            }
        }
        
        let shard = self.get_shard(user_id);
        let mut connections = shard.write();
        
        // Check per-shard limit
        let shard_size: usize = connections.values().map(|v| v.len()).sum();
        if shard_size >= self.max_connections_per_shard {
            warn!(
                shard_index = self.get_shard_index(user_id),
                shard_size = shard_size,
                "Shard connection limit reached"
            );
            return Err("Shard connection limit reached".to_string());
        }
        
        connections
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push((session_id.clone(), addr));
        
        // Increment total count
        *self.total_connections.write() += 1;
        
        info!(
            user_id = user_id,
            session_id = %session_id,
            shard_index = self.get_shard_index(user_id),
            total_connections = *self.total_connections.read(),
            "Connection registered"
        );
        
        // Broadcast user online status
        self.broadcast_user_status(user_id, true).await;
        
        Ok(())
    }
    
    /// Unregister a connection
    pub async fn unregister_connection(&self, user_id: i32, session_id: &str) {
        let shard = self.get_shard(user_id);
        let mut connections = shard.write();
        
        if let Some(user_connections) = connections.get_mut(&user_id) {
            user_connections.retain(|(sid, _)| sid != session_id);
            
            if user_connections.is_empty() {
                connections.remove(&user_id);
                
                // Decrement total count
                *self.total_connections.write() -= 1;
                
                info!(
                    user_id = user_id,
                    session_id = %session_id,
                    shard_index = self.get_shard_index(user_id),
                    total_connections = *self.total_connections.read(),
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
    pub fn is_user_online(&self, user_id: i32) -> bool {
        let shard = self.get_shard(user_id);
        let connections = shard.read();
        connections.contains_key(&user_id)
    }
    
    /// Send message to user
    pub async fn send_to_user(&self, user_id: i32, message: WsMessage) -> Result<(), String> {
        let shard = self.get_shard(user_id);
        let connections = shard.read();
        
        if let Some(user_connections) = connections.get(&user_id) {
            // Send to all user's connections
            for (session_id, addr) in user_connections {
                debug!(
                    user_id = user_id,
                    session_id = %session_id,
                    "Sending message to connection"
                );
                
                addr.do_send(crate::websocket::SendMessage(message.clone()));
            }
            
            Ok(())
        } else {
            Err(format!("User {} not connected", user_id))
        }
    }
    
    /// Deliver offline messages to user
    pub async fn deliver_offline_messages(&self, user_id: i32, addr: Addr<WsSession>) {
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
            
            addr.do_send(crate::websocket::SendMessage(ws_msg));
            
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
    
    /// Get total online user count
    pub fn online_count(&self) -> usize {
        *self.total_connections.read()
    }
    
    /// Get connection count for user
    pub fn user_connection_count(&self, user_id: i32) -> usize {
        let shard = self.get_shard(user_id);
        let connections = shard.read();
        connections.get(&user_id).map(|v| v.len()).unwrap_or(0)
    }
    
    /// Get shard statistics
    pub fn get_shard_stats(&self) -> Vec<ShardStats> {
        self.shards
            .iter()
            .enumerate()
            .map(|(index, shard)| {
                let connections = shard.read();
                let user_count = connections.len();
                let connection_count: usize = connections.values().map(|v| v.len()).sum();
                
                ShardStats {
                    shard_index: index,
                    user_count,
                    connection_count,
                }
            })
            .collect()
    }
    
    /// Get overall statistics
    pub fn get_stats(&self) -> ConnectionManagerStats {
        let shard_stats = self.get_shard_stats();
        let total_users: usize = shard_stats.iter().map(|s| s.user_count).sum();
        let total_connections = *self.total_connections.read();
        
        ConnectionManagerStats {
            total_users,
            total_connections,
            shard_count: self.shard_count,
            max_connections: self.max_connections_per_shard * self.shard_count,
            shard_stats,
        }
    }
}

/// Shard statistics
#[derive(Debug, Clone)]
pub struct ShardStats {
    pub shard_index: usize,
    pub user_count: usize,
    pub connection_count: usize,
}

/// Connection manager statistics
#[derive(Debug, Clone)]
pub struct ConnectionManagerStats {
    pub total_users: usize,
    pub total_connections: usize,
    pub shard_count: usize,
    pub max_connections: usize,
    pub shard_stats: Vec<ShardStats>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shard_distribution() {
        let cache = Arc::new(OfflineMessageCache::new(100));
        let manager = ShardedConnectionManager::new(cache, 256, 100_000);
        
        // Test that user IDs are evenly distributed
        let mut shard_counts = vec![0; 256];
        for user_id in 0..10000 {
            let index = manager.get_shard_index(user_id);
            shard_counts[index] += 1;
        }
        
        // Check that distribution is relatively even
        let avg = 10000 / 256;
        for count in shard_counts {
            assert!(count > avg - 10 && count < avg + 10);
        }
    }
    
    #[test]
    fn test_connection_limits() {
        let cache = Arc::new(OfflineMessageCache::new(100));
        let manager = ShardedConnectionManager::new(cache, 4, 100);
        
        assert_eq!(manager.max_connections_per_shard, 25);
    }
    
    #[tokio::test]
    async fn test_online_count() {
        let cache = Arc::new(OfflineMessageCache::new(100));
        let manager = ShardedConnectionManager::with_defaults(cache);
        
        assert_eq!(manager.online_count(), 0);
    }
}
