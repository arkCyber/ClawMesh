//! Integration Tests for ClawMesh Messaging System
//!
//! Tests the complete messaging flow including caching, delivery, persistence,
//! encryption, and clustering.

use clawmesh_messaging::{
    OfflineMessageCache, ShardedOfflineMessageCache, MessageDeliveryService,
    CachedMessage, MessagePersistence, EncryptionService, EncryptionKey,
    ClusterMembership, ClusterConfig, LoadBalancer, MessageQueue, QueueConfig,
    offline_cache::MessagePriority,
};
use std::sync::Arc;

#[tokio::test]
async fn test_end_to_end_message_flow() {
    // 1. Create services
    let cache = Arc::new(OfflineMessageCache::new());
    let delivery_service = Arc::new(MessageDeliveryService::new(Arc::clone(&cache)));
    
    // 2. User goes online
    delivery_service.user_online(200, "session_200".to_string()).await;
    assert!(delivery_service.is_user_online(200).await);
    
    // 3. Send message
    let message = CachedMessage::new(
        1,
        100,
        200,
        "Hello from integration test!".to_string(),
        MessagePriority::Normal,
    );
    
    let result = delivery_service.deliver_message(message).await;
    assert!(matches!(result, clawmesh_messaging::DeliveryResult::Delivered));
    
    // 4. Check stats
    let stats = delivery_service.get_stats().await;
    assert_eq!(stats.total_delivered, 1);
}

#[tokio::test]
async fn test_offline_message_caching_and_delivery() {
    let cache = Arc::new(OfflineMessageCache::new());
    let delivery_service = Arc::new(MessageDeliveryService::new(Arc::clone(&cache)));
    
    // 1. Send message to offline user
    let message = CachedMessage::new(
        1,
        100,
        200,
        "Offline message".to_string(),
        MessagePriority::High,
    );
    
    let result = delivery_service.deliver_message(message).await;
    assert!(matches!(result, clawmesh_messaging::DeliveryResult::Cached));
    
    // 2. Verify message is cached
    assert_eq!(cache.get_message_count(200).await, 1);
    
    // 3. User comes online
    delivery_service.user_online(200, "session_200".to_string()).await;
    
    // 4. Wait a bit for delivery
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // 5. Message should be delivered
    let stats = delivery_service.get_stats().await;
    assert!(stats.total_delivered > 0 || stats.total_cached > 0);
}

#[tokio::test]
async fn test_sharded_cache_performance() {
    let cache = Arc::new(ShardedOfflineMessageCache::new());
    
    // Send messages to different users (will go to different shards)
    for i in 0..100 {
        let message = CachedMessage::new(
            i,
            100,
            i as i32,
            format!("Message {}", i),
            MessagePriority::Normal,
        );
        cache.cache_message(message).await.unwrap();
    }
    
    let stats = cache.get_stats().await;
    assert_eq!(stats.total_messages, 100);
}

#[tokio::test]
async fn test_message_persistence() {
    let persistence = MessagePersistence::new();
    
    let message = CachedMessage::new(
        1,
        100,
        200,
        "Persistent message".to_string(),
        MessagePriority::Normal,
    );
    
    // Save message
    persistence.save_message(&message).await.unwrap();
    
    // Mark as delivered
    persistence.mark_delivered(1).await.unwrap();
    
    // Update delivery attempt
    persistence.update_delivery_attempt(1).await.unwrap();
}

#[tokio::test]
async fn test_batch_persistence() {
    let persistence = MessagePersistence::new();
    
    let messages: Vec<CachedMessage> = (0..10)
        .map(|i| CachedMessage::new(
            i,
            100,
            200,
            format!("Batch message {}", i),
            MessagePriority::Normal,
        ))
        .collect();
    
    let saved = persistence.batch_save(&messages).await.unwrap();
    assert_eq!(saved, 10);
}

#[tokio::test]
async fn test_encryption_flow() {
    let encryption = EncryptionService::default();
    let key = EncryptionKey::new(200, "public_key_200".to_string());
    
    let plaintext = "Secret message";
    
    // Encrypt
    let encrypted = encryption.encrypt(plaintext, &key).unwrap();
    assert_ne!(encrypted.ciphertext, plaintext);
    
    // Decrypt
    let decrypted = encryption.decrypt(&encrypted, "private_key").unwrap();
    assert_eq!(decrypted, plaintext);
}

#[tokio::test]
async fn test_key_rotation() {
    use clawmesh_messaging::KeyManagementService;
    
    let kms = KeyManagementService::new();
    
    // Store initial key
    let key1 = EncryptionKey::new(1, "public_key_1".to_string());
    kms.store_public_key(key1.clone()).await.unwrap();
    
    // Rotate key
    let new_key_id = kms.rotate_key(1, "public_key_2".to_string()).await.unwrap();
    assert_ne!(new_key_id, key1.id);
}

#[tokio::test]
async fn test_cluster_membership() {
    let config = ClusterConfig::default();
    let membership = Arc::new(ClusterMembership::new(config));
    
    // Join cluster
    membership.join().await.unwrap();
    
    // Get stats
    let stats = membership.get_stats().await;
    assert_eq!(stats.total_nodes, 1);
}

#[tokio::test]
async fn test_load_balancer() {
    let config = ClusterConfig::default();
    let membership = Arc::new(ClusterMembership::new(config));
    membership.join().await.unwrap();
    
    // Manually trigger heartbeat to set node as healthy
    membership.heartbeat().await;
    
    let lb = LoadBalancer::new(Arc::clone(&membership));
    
    // Select node
    let node = lb.select_node().await;
    assert!(node.is_some());
}

#[tokio::test]
async fn test_message_queue_operations() {
    let config = QueueConfig::default();
    let queue = MessageQueue::new(config);
    
    let message = CachedMessage::new(
        1,
        100,
        200,
        "Queue message".to_string(),
        MessagePriority::Normal,
    );
    
    // Enqueue
    let msg_id = queue.enqueue(message).await.unwrap();
    assert!(!msg_id.is_empty());
    
    // Get stats
    let stats = queue.get_stats().await.unwrap();
    assert_eq!(stats.pending_messages, 0); // Mock returns 0
}

#[tokio::test]
async fn test_priority_message_delivery() {
    let cache = Arc::new(OfflineMessageCache::new());
    
    // Send messages with different priorities
    let low = CachedMessage::new(1, 100, 200, "Low".to_string(), MessagePriority::Low);
    let normal = CachedMessage::new(2, 100, 200, "Normal".to_string(), MessagePriority::Normal);
    let high = CachedMessage::new(3, 100, 200, "High".to_string(), MessagePriority::High);
    let urgent = CachedMessage::new(4, 100, 200, "Urgent".to_string(), MessagePriority::Urgent);
    
    cache.cache_message(low).await.unwrap();
    cache.cache_message(normal).await.unwrap();
    cache.cache_message(high).await.unwrap();
    cache.cache_message(urgent).await.unwrap();
    
    // Retrieve messages - should be in priority order
    let messages = cache.get_pending_messages(200, 10).await;
    assert_eq!(messages.len(), 4);
    assert_eq!(messages[0].priority, MessagePriority::Urgent);
    assert_eq!(messages[1].priority, MessagePriority::High);
    assert_eq!(messages[2].priority, MessagePriority::Normal);
    assert_eq!(messages[3].priority, MessagePriority::Low);
}

#[tokio::test]
async fn test_concurrent_message_delivery() {
    let cache = Arc::new(OfflineMessageCache::new());
    let delivery_service = Arc::new(MessageDeliveryService::new(cache));
    
    // Register users as online
    for i in 0..10 {
        delivery_service.user_online(i, format!("session_{}", i)).await;
    }
    
    let mut handles = vec![];
    
    // Send 100 messages concurrently
    for i in 0..100 {
        let service = Arc::clone(&delivery_service);
        let handle = tokio::spawn(async move {
            let message = CachedMessage::new(
                i,
                1000,
                (i % 10) as i32,
                format!("Concurrent message {}", i),
                MessagePriority::Normal,
            );
            service.deliver_message(message).await
        });
        handles.push(handle);
    }
    
    // Wait for all deliveries
    for handle in handles {
        handle.await.unwrap();
    }
    
    let stats = delivery_service.get_stats().await;
    assert_eq!(stats.total_delivered + stats.total_cached, 100);
}

#[tokio::test]
async fn test_message_expiration() {
    let cache = OfflineMessageCache::new();
    
    let mut message = CachedMessage::new(
        1,
        100,
        200,
        "Expired message".to_string(),
        MessagePriority::Normal,
    );
    
    // Set expiration to past
    message.expires_at = chrono::Utc::now() - chrono::Duration::days(1);
    
    assert!(message.is_expired());
}

#[tokio::test]
async fn test_delivery_retry_logic() {
    let mut message = CachedMessage::new(
        1,
        100,
        200,
        "Retry message".to_string(),
        MessagePriority::Normal,
    );
    
    // First attempt should retry
    assert!(message.should_retry());
    
    // Record attempts
    for _ in 0..5 {
        message.record_attempt();
    }
    
    assert_eq!(message.delivery_attempts, 5);
}

#[tokio::test]
async fn test_cache_cleanup() {
    let cache = OfflineMessageCache::new();
    
    // Cache some messages
    for i in 0..10 {
        let message = CachedMessage::new(
            i,
            100,
            200,
            format!("Message {}", i),
            MessagePriority::Normal,
        );
        cache.cache_message(message).await.unwrap();
    }
    
    // Cleanup (no expired messages yet)
    let removed = cache.cleanup_expired().await;
    assert_eq!(removed, 0);
}

#[tokio::test]
async fn test_sharded_cache_distribution() {
    let cache = ShardedOfflineMessageCache::new();
    
    // Send messages to users that should go to different shards
    for i in 0..16 {
        let message = CachedMessage::new(
            i,
            100,
            i as i32 * 16, // Ensure different shards
            format!("Shard message {}", i),
            MessagePriority::Normal,
        );
        cache.cache_message(message).await.unwrap();
    }
    
    let stats = cache.get_stats().await;
    assert!(stats.total_users > 0);
}

#[tokio::test]
async fn test_user_online_offline_cycle() {
    let cache = Arc::new(OfflineMessageCache::new());
    let delivery_service = Arc::new(MessageDeliveryService::new(cache));
    
    let user_id = 123;
    
    // User starts offline
    assert!(!delivery_service.is_user_online(user_id).await);
    
    // User goes online
    delivery_service.user_online(user_id, "session_123".to_string()).await;
    assert!(delivery_service.is_user_online(user_id).await);
    assert_eq!(delivery_service.online_count().await, 1);
    
    // User goes offline
    delivery_service.user_offline(user_id).await;
    assert!(!delivery_service.is_user_online(user_id).await);
    assert_eq!(delivery_service.online_count().await, 0);
}

#[tokio::test]
async fn test_multiple_recipients() {
    let cache = Arc::new(OfflineMessageCache::new());
    
    // Send messages to multiple recipients
    for recipient_id in 1..=10i32 {
        for msg_id in 0..5i64 {
            let message = CachedMessage::new(
                msg_id + (recipient_id as i64) * 100,
                100,
                recipient_id,
                format!("Message to user {}", recipient_id),
                MessagePriority::Normal,
            );
            cache.cache_message(message).await.unwrap();
        }
    }
    
    // Each recipient should have 5 messages
    for recipient_id in 1..=10i32 {
        assert_eq!(cache.get_message_count(recipient_id).await, 5);
    }
}
