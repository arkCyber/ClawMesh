//! ClawMesh Messaging System
//! 
//! Provides P2P file transfer, real-time messaging, end-to-end encryption,
//! and offline message caching with support for 100,000+ concurrent users
//!
//! Note: Group/Community features use Lemmy's existing Community system

// Core messaging features (use Lemmy's Community/PrivateMessage)
pub mod direct;
pub mod offline_cache;
pub mod delivery;
pub mod persistence;
pub mod sharded_cache;

// Real implementations (not mocks)
pub mod redis_queue;
pub mod ring_encryption;
pub mod websocket;
pub mod sharded_connection_manager;

// ClawMesh unique features
pub mod cluster;
pub mod p2p_transfer;
pub mod p2p_signaling;
pub mod file_storage;
pub mod p2p_disk_storage;
pub mod errors;

// Re-exports
pub use direct::{DirectMessage, DirectMessageForm, Conversation, ConversationSummary};
pub use offline_cache::{OfflineMessageCache, CachedMessage, CacheStats};
pub use delivery::{MessageDeliveryService, DeliveryResult, DeliveryStats};
pub use persistence::MessagePersistence;
pub use sharded_cache::ShardedOfflineMessageCache;
pub use redis_queue::RedisMessageQueue;
pub use ring_encryption::{RingEncryptionService, RingKeyManagementService};
pub use cluster::{ClusterMembership, ClusterConfig, ClusterNode, LoadBalancer};
pub use p2p_transfer::{P2PTransferService, P2PConfig, FileTransfer, FileChunk, TransferMode, TransferStatus, TransferStats};
pub use p2p_signaling::{SignalingServer, SignalingMessage, WebSocketSession};
pub use file_storage::{FileStorageService, StorageConfig, StoredFile, StorageStats};
pub use errors::{MessagingError, P2PError, StorageError, EncryptionError};
pub use p2p_disk_storage::{DiskChunkStorage, StorageStats as DiskStorageStats};
pub use websocket::{WsMessage, ClientMessage, WsSession, ConnectionManager, websocket_handler};
pub use sharded_connection_manager::{ShardedConnectionManager, ShardStats, ConnectionManagerStats};

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Message priority level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessagePriority {
    /// Low priority
    Low,
    /// Normal priority
    Normal,
    /// High priority
    High,
    /// Urgent priority
    Urgent,
}

/// Message status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageStatus {
    /// Message sent
    Sent,
    /// Message delivered
    Delivered,
    /// Message read
    Read,
    /// Message failed
    Failed,
}

/// Messaging configuration
#[derive(Debug, Clone)]
pub struct MessagingConfig {
    /// Maximum group size
    pub max_group_size: usize,
    /// Maximum message length
    pub max_message_length: usize,
    /// Message retention days
    pub message_retention_days: i32,
    /// Enable message encryption
    pub enable_encryption: bool,
}

impl Default for MessagingConfig {
    fn default() -> Self {
        Self {
            max_group_size: 1000,
            max_message_length: 10_000,
            message_retention_days: 365,
            enable_encryption: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_priority() {
        let priority = MessagePriority::High;
        assert_eq!(priority, MessagePriority::High);
    }

    #[test]
    fn test_default_config() {
        let config = MessagingConfig::default();
        assert_eq!(config.max_group_size, 1000);
        assert_eq!(config.max_message_length, 10_000);
    }
}
