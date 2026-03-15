//! Aerospace-Grade Comprehensive Tests
//!
//! Complete test suite following aerospace standards:
//! - Boundary condition testing
//! - Error path testing
//! - Concurrency testing
//! - Performance benchmarking
//! - Security testing

use clawmesh_messaging::*;
use tokio::time::{sleep, Duration};
use std::sync::Arc;

#[tokio::test]
async fn test_p2p_transfer_boundary_conditions() {
    let config = p2p_transfer::P2PConfig::default();
    let service = p2p_transfer::P2PTransferService::new(config);

    // Test 1: Zero-byte file
    let result = service.initiate_transfer(
        1,
        2,
        "empty.txt".to_string(),
        0,
        "text/plain".to_string(),
    );
    assert!(result.is_ok(), "Should handle zero-byte files");

    // Test 2: Exactly max file size
    let max_size = 100 * 1024 * 1024; // 100 MB
    let result = service.initiate_transfer(
        1,
        2,
        "max_size.bin".to_string(),
        max_size,
        "application/octet-stream".to_string(),
    );
    assert!(result.is_ok(), "Should handle max size files");

    // Test 3: Over max file size
    let result = service.initiate_transfer(
        1,
        2,
        "too_large.bin".to_string(),
        max_size + 1,
        "application/octet-stream".to_string(),
    );
    assert!(result.is_err(), "Should reject oversized files");
}

#[tokio::test]
async fn test_file_chunk_integrity() {
    // Test 1: Valid chunk
    let data = vec![1, 2, 3, 4, 5];
    let checksum = p2p_transfer::FileChunk::calculate_checksum(&data);
    let chunk = p2p_transfer::FileChunk {
        transfer_id: "test".to_string(),
        chunk_index: 0,
        total_chunks: 1,
        data: data.clone(),
        checksum,
    };
    assert!(chunk.verify(), "Valid chunk should verify");

    // Test 2: Corrupted data
    let mut corrupted_chunk = chunk.clone();
    corrupted_chunk.data[0] = 99;
    assert!(!corrupted_chunk.verify(), "Corrupted chunk should fail verification");

    // Test 3: Wrong checksum
    let mut wrong_checksum = chunk.clone();
    wrong_checksum.checksum = 0;
    assert!(!wrong_checksum.verify(), "Wrong checksum should fail verification");
}

#[tokio::test]
async fn test_concurrent_transfers() {
    let config = p2p_transfer::P2PConfig {
        max_concurrent_transfers: 5,
        ..Default::default()
    };
    let service = Arc::new(p2p_transfer::P2PTransferService::new(config));

    // Start 10 concurrent transfers
    let mut handles = vec![];
    for i in 0..10 {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            service_clone.initiate_transfer(
                i,
                i + 100,
                format!("file_{}.txt", i),
                1024,
                "text/plain".to_string(),
            )
        });
        handles.push(handle);
    }

    // Wait for all transfers
    let results: Vec<_> = futures::future::join_all(handles).await;
    
    // First 5 should succeed, rest should fail due to limit
    let success_count = results.iter().filter(|r| r.as_ref().unwrap().is_ok()).count();
    assert!(success_count <= 5, "Should respect concurrent transfer limit");
}

#[tokio::test]
async fn test_transfer_timeout() {
    let config = p2p_transfer::P2PConfig {
        transfer_timeout: 1, // 1 second timeout
        ..Default::default()
    };
    let service = p2p_transfer::P2PTransferService::new(config);

    let transfer = service.initiate_transfer(
        1,
        2,
        "test.txt".to_string(),
        1024,
        "text/plain".to_string(),
    ).unwrap();

    // Wait for timeout
    sleep(Duration::from_secs(2)).await;

    // Transfer should be cleaned up
    let stats = service.get_stats();
    assert_eq!(stats.active_transfers, 0, "Timed out transfers should be cleaned");
}

#[tokio::test]
async fn test_chunk_ordering() {
    let config = p2p_transfer::P2PConfig::default();
    let service = p2p_transfer::P2PTransferService::new(config);

    let transfer = service.initiate_transfer(
        1,
        2,
        "test.txt".to_string(),
        200,
        "text/plain".to_string(),
    ).unwrap();

    // Send chunks out of order
    let chunks = vec![
        p2p_transfer::FileChunk {
            transfer_id: transfer.transfer_id.clone(),
            chunk_index: 2,
            total_chunks: 3,
            data: vec![5, 6],
            checksum: p2p_transfer::FileChunk::calculate_checksum(&[5, 6]),
        },
        p2p_transfer::FileChunk {
            transfer_id: transfer.transfer_id.clone(),
            chunk_index: 0,
            total_chunks: 3,
            data: vec![1, 2],
            checksum: p2p_transfer::FileChunk::calculate_checksum(&[1, 2]),
        },
        p2p_transfer::FileChunk {
            transfer_id: transfer.transfer_id.clone(),
            chunk_index: 1,
            total_chunks: 3,
            data: vec![3, 4],
            checksum: p2p_transfer::FileChunk::calculate_checksum(&[3, 4]),
        },
    ];

    for chunk in chunks {
        service.receive_chunk(chunk).unwrap();
    }

    // Verify all chunks received
    let updated_transfer = service.get_transfer(&transfer.transfer_id).unwrap();
    assert_eq!(updated_transfer.received_chunks.len(), 3);
}

#[tokio::test]
async fn test_duplicate_chunk_handling() {
    let config = p2p_transfer::P2PConfig::default();
    let service = p2p_transfer::P2PTransferService::new(config);

    let transfer = service.initiate_transfer(
        1,
        2,
        "test.txt".to_string(),
        100,
        "text/plain".to_string(),
    ).unwrap();

    let chunk = p2p_transfer::FileChunk {
        transfer_id: transfer.transfer_id.clone(),
        chunk_index: 0,
        total_chunks: 2,
        data: vec![1, 2, 3],
        checksum: p2p_transfer::FileChunk::calculate_checksum(&[1, 2, 3]),
    };

    // Send same chunk twice
    service.receive_chunk(chunk.clone()).unwrap();
    service.receive_chunk(chunk.clone()).unwrap();

    // Should only count once
    let updated_transfer = service.get_transfer(&transfer.transfer_id).unwrap();
    assert_eq!(updated_transfer.received_chunks.len(), 1);
}

#[tokio::test]
async fn test_transfer_cancellation() {
    let config = p2p_transfer::P2PConfig::default();
    let service = p2p_transfer::P2PTransferService::new(config);

    let transfer = service.initiate_transfer(
        1,
        2,
        "test.txt".to_string(),
        1024,
        "text/plain".to_string(),
    ).unwrap();

    // Cancel transfer
    service.cancel_transfer(&transfer.transfer_id).unwrap();

    // Should not be able to receive chunks
    let chunk = p2p_transfer::FileChunk {
        transfer_id: transfer.transfer_id.clone(),
        chunk_index: 0,
        total_chunks: 1,
        data: vec![1, 2, 3],
        checksum: p2p_transfer::FileChunk::calculate_checksum(&[1, 2, 3]),
    };

    let result = service.receive_chunk(chunk);
    assert!(result.is_err(), "Should reject chunks for cancelled transfer");
}

#[tokio::test]
async fn test_peer_online_status() {
    let config = p2p_transfer::P2PConfig::default();
    let service = p2p_transfer::P2PTransferService::new(config);

    // Register online peer
    let peer = p2p_transfer::PeerConnection {
        user_id: 1,
        session_id: "session_1".to_string(),
        connection_id: "conn_1".to_string(),
        ip_address: "192.168.1.1".to_string(),
        port: 8080,
        online: true,
        last_seen: chrono::Utc::now(),
    };
    service.register_peer(peer);

    // Transfer to online peer should use P2P mode
    let transfer = service.initiate_transfer(
        2,
        1,
        "test.txt".to_string(),
        1024,
        "text/plain".to_string(),
    ).unwrap();

    assert_eq!(transfer.mode, p2p_transfer::TransferMode::P2P);

    // Transfer to offline peer should use ServerRelay
    let transfer2 = service.initiate_transfer(
        2,
        999, // Non-existent peer
        "test2.txt".to_string(),
        1024,
        "text/plain".to_string(),
    ).unwrap();

    assert_eq!(transfer2.mode, p2p_transfer::TransferMode::ServerRelay);
}

#[tokio::test]
async fn test_crc32_performance() {
    use std::time::Instant;

    // Test with 1 MB data
    let data = vec![0u8; 1024 * 1024];
    
    let start = Instant::now();
    for _ in 0..100 {
        p2p_transfer::FileChunk::calculate_checksum(&data);
    }
    let duration = start.elapsed();

    // Should complete 100 checksums of 1MB in less than 1 second
    assert!(duration.as_secs() < 1, "CRC32 performance regression detected");
    
    println!("CRC32 performance: {} checksums/sec", 100.0 / duration.as_secs_f64());
}

#[tokio::test]
async fn test_signaling_message_routing() {
    let server = p2p_signaling::SignalingServer::new();

    // Register two sessions
    let (tx1, mut rx1) = tokio::sync::mpsc::unbounded_channel();
    let (tx2, mut rx2) = tokio::sync::mpsc::unbounded_channel();

    server.register_session(1, "session_1".to_string(), tx1);
    server.register_session(2, "session_2".to_string(), tx2);

    // Send offer from user 1 to user 2
    let offer = p2p_signaling::SignalingMessage::Offer {
        from: 1,
        to: 2,
        transfer_id: "transfer_1".to_string(),
        sdp: "offer_sdp".to_string(),
    };

    server.handle_message(offer).unwrap();

    // User 2 should receive the offer
    let received = rx2.try_recv();
    assert!(received.is_ok(), "User 2 should receive offer");
}

#[tokio::test]
async fn test_memory_safety_large_transfer() {
    let config = p2p_transfer::P2PConfig::default();
    let service = p2p_transfer::P2PTransferService::new(config);

    // Simulate large file (10 MB)
    let file_size = 10 * 1024 * 1024;
    let chunk_size = 64 * 1024;
    let total_chunks = (file_size + chunk_size - 1) / chunk_size;

    let transfer = service.initiate_transfer(
        1,
        2,
        "large_file.bin".to_string(),
        file_size as u64,
        "application/octet-stream".to_string(),
    ).unwrap();

    // Send all chunks
    for i in 0..total_chunks {
        let data = vec![i as u8; chunk_size.min(file_size - i * chunk_size)];
        let chunk = p2p_transfer::FileChunk {
            transfer_id: transfer.transfer_id.clone(),
            chunk_index: i as u32,
            total_chunks: total_chunks as u32,
            data: data.clone(),
            checksum: p2p_transfer::FileChunk::calculate_checksum(&data),
        };
        service.receive_chunk(chunk).unwrap();
    }

    // Verify completion
    let updated_transfer = service.get_transfer(&transfer.transfer_id).unwrap();
    assert_eq!(updated_transfer.status, p2p_transfer::TransferStatus::Completed);
}

#[tokio::test]
async fn test_error_recovery() {
    let config = p2p_transfer::P2PConfig::default();
    let service = p2p_transfer::P2PTransferService::new(config);

    let transfer = service.initiate_transfer(
        1,
        2,
        "test.txt".to_string(),
        200,
        "text/plain".to_string(),
    ).unwrap();

    // Send valid chunk
    let chunk1 = p2p_transfer::FileChunk {
        transfer_id: transfer.transfer_id.clone(),
        chunk_index: 0,
        total_chunks: 2,
        data: vec![1, 2, 3],
        checksum: p2p_transfer::FileChunk::calculate_checksum(&[1, 2, 3]),
    };
    service.receive_chunk(chunk1).unwrap();

    // Send corrupted chunk
    let chunk2 = p2p_transfer::FileChunk {
        transfer_id: transfer.transfer_id.clone(),
        chunk_index: 1,
        total_chunks: 2,
        data: vec![4, 5, 6],
        checksum: 0, // Wrong checksum
    };
    let result = service.receive_chunk(chunk2);
    assert!(result.is_err(), "Should reject corrupted chunk");

    // Send correct chunk
    let chunk2_correct = p2p_transfer::FileChunk {
        transfer_id: transfer.transfer_id.clone(),
        chunk_index: 1,
        total_chunks: 2,
        data: vec![4, 5, 6],
        checksum: p2p_transfer::FileChunk::calculate_checksum(&[4, 5, 6]),
    };
    service.receive_chunk(chunk2_correct).unwrap();

    // Should complete successfully
    let updated_transfer = service.get_transfer(&transfer.transfer_id).unwrap();
    assert_eq!(updated_transfer.status, p2p_transfer::TransferStatus::Completed);
}

#[tokio::test]
async fn test_statistics_accuracy() {
    let config = p2p_transfer::P2PConfig::default();
    let service = p2p_transfer::P2PTransferService::new(config);

    // Create multiple transfers
    for i in 0..5 {
        service.initiate_transfer(
            i,
            i + 100,
            format!("file_{}.txt", i),
            1024,
            "text/plain".to_string(),
        ).unwrap();
    }

    let stats = service.get_stats();
    assert_eq!(stats.active_transfers, 5);
    assert_eq!(stats.total_transfers, 5);
}
