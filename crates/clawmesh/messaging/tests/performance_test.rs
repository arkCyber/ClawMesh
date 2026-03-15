//! Performance Benchmarks for 100,000+ Concurrent Users
//!
//! Tests the messaging system's ability to handle high concurrency,
//! offline message caching, and delivery at scale.

use clawmesh_messaging::{
    OfflineMessageCache, MessageDeliveryService, CachedMessage,
    offline_cache::MessagePriority,
};
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Runtime;

/// Test 10,000 concurrent message deliveries
#[test]
fn bench_10k_concurrent_deliveries() {
    let rt = Runtime::new().unwrap();
    
    rt.block_on(async {
        let cache = Arc::new(OfflineMessageCache::new());
        let service = Arc::new(MessageDeliveryService::new(Arc::clone(&cache)));

        // Register 1,000 users as online
        for i in 0..1000 {
            service.user_online(i, format!("session_{}", i)).await;
        }

        let start = Instant::now();
        let mut handles = vec![];

        // Send 10,000 messages concurrently
        for i in 0..10_000 {
            let service_clone = Arc::clone(&service);
            let handle = tokio::spawn(async move {
                let message = CachedMessage::new(
                    i,
                    10_000 + (i % 100) as i32,
                    (i % 1000) as i32,
                    format!("Performance test message {}", i),
                    MessagePriority::Normal,
                );
                service_clone.deliver_message(message).await
            });
            handles.push(handle);
        }

        // Wait for all deliveries
        for handle in handles {
            handle.await.unwrap();
        }

        let elapsed = start.elapsed();
        let stats = service.get_stats().await;

        println!("\n=== 10K Concurrent Deliveries ===");
        println!("Total time: {:?}", elapsed);
        println!("Messages/sec: {:.2}", 10_000.0 / elapsed.as_secs_f64());
        println!("Delivered: {}", stats.total_delivered);
        println!("Cached: {}", stats.total_cached);
        println!("Failed: {}", stats.total_failed);
        println!("Avg delivery time: {:.2}ms", stats.avg_delivery_time_ms);

        assert_eq!(stats.total_delivered + stats.total_cached, 10_000);
    });
}

/// Test 100,000 users with offline message caching
#[test]
fn bench_100k_users_offline_cache() {
    let rt = Runtime::new().unwrap();
    
    rt.block_on(async {
        let cache = Arc::new(OfflineMessageCache::new());
        let start = Instant::now();

        // Cache 100,000 messages for 10,000 different users
        let mut handles = vec![];
        for i in 0..100_000 {
            let cache_clone = Arc::clone(&cache);
            let handle = tokio::spawn(async move {
                let message = CachedMessage::new(
                    i,
                    (i % 50_000) as i32,
                    (i % 10_000) as i32,
                    format!("Cached message {}", i),
                    if i % 10 == 0 { MessagePriority::High } else { MessagePriority::Normal },
                );
                cache_clone.cache_message(message).await
            });
            handles.push(handle);
        }

        // Wait for all caching operations
        let mut success = 0;
        for handle in handles {
            if handle.await.unwrap().is_ok() {
                success += 1;
            }
        }

        let elapsed = start.elapsed();
        let stats = cache.get_stats().await;

        println!("\n=== 100K Users Offline Cache ===");
        println!("Total time: {:?}", elapsed);
        println!("Cache ops/sec: {:.2}", 100_000.0 / elapsed.as_secs_f64());
        println!("Successful caches: {}", success);
        println!("Total cached messages: {}", stats.total_messages);
        println!("Total users: {}", stats.total_users);
        println!("Total cache size: {} bytes", stats.total_size);

        assert!(success > 90_000, "Should cache at least 90% of messages");
    });
}

/// Test message retrieval performance
#[test]
fn bench_message_retrieval() {
    let rt = Runtime::new().unwrap();
    
    rt.block_on(async {
        let cache = Arc::new(OfflineMessageCache::new());

        // Cache 1,000 messages for user 1
        for i in 0..1000 {
            let message = CachedMessage::new(
                i,
                100,
                1,
                format!("Message {}", i),
                MessagePriority::Normal,
            );
            cache.cache_message(message).await.unwrap();
        }

        let start = Instant::now();
        let messages = cache.get_pending_messages(1, 1000).await;
        let elapsed = start.elapsed();

        println!("\n=== Message Retrieval ===");
        println!("Retrieved {} messages in {:?}", messages.len(), elapsed);
        println!("Retrieval rate: {:.2} msg/ms", messages.len() as f64 / elapsed.as_millis() as f64);

        assert_eq!(messages.len(), 1000);
    });
}

/// Test concurrent user online/offline operations
#[test]
fn bench_user_presence_updates() {
    let rt = Runtime::new().unwrap();
    
    rt.block_on(async {
        let cache = Arc::new(OfflineMessageCache::new());
        let service = Arc::new(MessageDeliveryService::new(cache));

        let start = Instant::now();
        let mut handles = vec![];

        // Simulate 10,000 users going online/offline
        for i in 0..10_000 {
            let service_clone = Arc::clone(&service);
            let handle = tokio::spawn(async move {
                service_clone.user_online(i, format!("session_{}", i)).await;
                tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
                service_clone.user_offline(i).await;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let elapsed = start.elapsed();

        println!("\n=== User Presence Updates ===");
        println!("Total time: {:?}", elapsed);
        println!("Updates/sec: {:.2}", 20_000.0 / elapsed.as_secs_f64());
        println!("Final online count: {}", service.online_count().await);

        assert_eq!(service.online_count().await, 0);
    });
}

/// Test priority message ordering
#[test]
fn bench_priority_ordering() {
    let rt = Runtime::new().unwrap();
    
    rt.block_on(async {
        let cache = Arc::new(OfflineMessageCache::new());

        // Cache messages with different priorities
        for i in 0..1000 {
            let priority = match i % 4 {
                0 => MessagePriority::Urgent,
                1 => MessagePriority::High,
                2 => MessagePriority::Normal,
                _ => MessagePriority::Low,
            };
            
            let message = CachedMessage::new(
                i,
                100,
                1,
                format!("Message {}", i),
                priority,
            );
            cache.cache_message(message).await.unwrap();
        }

        let start = Instant::now();
        let messages = cache.get_pending_messages(1, 100).await;
        let elapsed = start.elapsed();

        println!("\n=== Priority Ordering ===");
        println!("Retrieved {} messages in {:?}", messages.len(), elapsed);
        
        // Verify priority ordering
        let mut prev_priority = MessagePriority::Urgent;
        for msg in &messages {
            assert!(msg.priority >= prev_priority || msg.priority == MessagePriority::Urgent);
            prev_priority = msg.priority;
        }

        println!("Priority ordering verified");
    });
}

/// Stress test: Maximum throughput
#[test]
#[ignore] // Run with --ignored flag for stress testing
fn stress_test_max_throughput() {
    let rt = Runtime::new().unwrap();
    
    rt.block_on(async {
        let cache = Arc::new(OfflineMessageCache::new());
        let service = Arc::new(MessageDeliveryService::new(Arc::clone(&cache)));

        // Register 10,000 users as online
        for i in 0..10_000 {
            service.user_online(i, format!("session_{}", i)).await;
        }

        let start = Instant::now();
        let mut handles = vec![];

        // Send 1,000,000 messages
        for i in 0..1_000_000 {
            let service_clone = Arc::clone(&service);
            let handle = tokio::spawn(async move {
                let message = CachedMessage::new(
                    i,
                    (i % 100_000) as i32,
                    (i % 10_000) as i32,
                    format!("Stress test message {}", i),
                    MessagePriority::Normal,
                );
                service_clone.deliver_message(message).await
            });
            handles.push(handle);

            // Batch spawn to avoid overwhelming the runtime
            if handles.len() >= 10_000 {
                for h in handles.drain(..) {
                    h.await.unwrap();
                }
            }
        }

        // Wait for remaining tasks
        for handle in handles {
            handle.await.unwrap();
        }

        let elapsed = start.elapsed();
        let stats = service.get_stats().await;

        println!("\n=== STRESS TEST: 1M Messages ===");
        println!("Total time: {:?}", elapsed);
        println!("Messages/sec: {:.2}", 1_000_000.0 / elapsed.as_secs_f64());
        println!("Delivered: {}", stats.total_delivered);
        println!("Cached: {}", stats.total_cached);
        println!("Failed: {}", stats.total_failed);
        println!("Avg delivery time: {:.2}ms", stats.avg_delivery_time_ms);
        println!("Queue size: {}", stats.queue_size);
    });
}
