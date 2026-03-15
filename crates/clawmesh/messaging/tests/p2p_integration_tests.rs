//! P2P File Transfer Integration Tests

use clawmesh_messaging::{
    P2PTransferService, P2PConfig, FileChunk, TransferMode, TransferStatus,
    SignalingServer, SignalingMessage, PeerConnection,
    FileStorageService, StorageConfig,
};
use tokio::sync::mpsc;
use std::sync::Arc;

#[tokio::test]
async fn test_p2p_transfer_online_peers() {
    // Setup
    let p2p_service = Arc::new(P2PTransferService::new(P2PConfig::default()));
    
    // Register both peers as online
    p2p_service.register_peer(PeerConnection {
        user_id: 1,
        session_id: "session_1".to_string(),
        connection_id: "conn_1".to_string(),
        ip_address: "192.168.1.100".to_string(),
        port: 8080,
        online: true,
        last_seen: chrono::Utc::now(),
    });
    
    p2p_service.register_peer(PeerConnection {
        user_id: 2,
        session_id: "session_2".to_string(),
        connection_id: "conn_2".to_string(),
        ip_address: "192.168.1.101".to_string(),
        port: 8080,
        online: true,
        last_seen: chrono::Utc::now(),
    });
    
    // Initiate transfer
    let transfer = p2p_service.initiate_transfer(
        1,
        2,
        "document.pdf".to_string(),
        1024 * 1024, // 1 MB
        "application/pdf".to_string(),
    ).unwrap();
    
    // Should attempt P2P since both are online
    assert_eq!(transfer.mode, TransferMode::P2P);
    assert_eq!(transfer.status, TransferStatus::Negotiating);
}

#[tokio::test]
async fn test_p2p_transfer_offline_peer_fallback() {
    let p2p_service = Arc::new(P2PTransferService::new(P2PConfig::default()));
    
    // Only sender is online
    p2p_service.register_peer(PeerConnection {
        user_id: 1,
        session_id: "session_1".to_string(),
        connection_id: "conn_1".to_string(),
        ip_address: "192.168.1.100".to_string(),
        port: 8080,
        online: true,
        last_seen: chrono::Utc::now(),
    });
    
    // Initiate transfer to offline peer
    let transfer = p2p_service.initiate_transfer(
        1,
        2,
        "video.mp4".to_string(),
        10 * 1024 * 1024, // 10 MB
        "video/mp4".to_string(),
    ).unwrap();
    
    // Should use server relay
    assert_eq!(transfer.mode, TransferMode::ServerRelay);
    assert_eq!(transfer.status, TransferStatus::Pending);
}

#[tokio::test]
async fn test_chunked_file_transfer() {
    let p2p_service = Arc::new(P2PTransferService::new(P2PConfig {
        chunk_size: 1024, // 1 KB chunks for testing
        ..Default::default()
    }));
    
    // Initiate transfer
    let transfer = p2p_service.initiate_transfer(
        1,
        2,
        "test.bin".to_string(),
        5000, // 5 KB file
        "application/octet-stream".to_string(),
    ).unwrap();
    
    let transfer_id = transfer.transfer_id.clone();
    assert_eq!(transfer.total_chunks, 5); // 5000 / 1024 = 5 chunks
    
    // Send all chunks
    for i in 0..transfer.total_chunks {
        let data = vec![i as u8; 1024];
        let checksum = FileChunk::calculate_checksum(&data);
        
        let chunk = FileChunk {
            transfer_id: transfer_id.clone(),
            chunk_index: i,
            total_chunks: transfer.total_chunks,
            data,
            checksum,
        };
        
        p2p_service.receive_chunk(chunk).unwrap();
    }
    
    // Verify transfer completed
    let completed_transfer = p2p_service.get_transfer(&transfer_id).unwrap();
    assert_eq!(completed_transfer.status, TransferStatus::Completed);
    assert_eq!(completed_transfer.progress(), 1.0);
    assert!(completed_transfer.is_complete());
}

#[tokio::test]
async fn test_chunk_integrity_verification() {
    let p2p_service = Arc::new(P2PTransferService::new(P2PConfig::default()));
    
    let transfer = p2p_service.initiate_transfer(
        1, 2, "test.bin".to_string(), 1000, "application/octet-stream".to_string()
    ).unwrap();
    
    let data = vec![42u8; 1024];
    let wrong_checksum = 0xDEADBEEF; // Intentionally wrong
    
    let bad_chunk = FileChunk {
        transfer_id: transfer.transfer_id.clone(),
        chunk_index: 0,
        total_chunks: 1,
        data,
        checksum: wrong_checksum,
    };
    
    // Should fail verification
    let result = p2p_service.receive_chunk(bad_chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("checksum"));
}

#[tokio::test]
async fn test_transfer_progress_tracking() {
    let p2p_service = Arc::new(P2PTransferService::new(P2PConfig {
        chunk_size: 1000,
        ..Default::default()
    }));
    
    let transfer = p2p_service.initiate_transfer(
        1, 2, "test.bin".to_string(), 10000, "application/octet-stream".to_string()
    ).unwrap();
    
    let transfer_id = transfer.transfer_id.clone();
    
    // Send first 5 chunks (50%)
    for i in 0..5 {
        let data = vec![i as u8; 1000];
        let checksum = FileChunk::calculate_checksum(&data);
        
        let chunk = FileChunk {
            transfer_id: transfer_id.clone(),
            chunk_index: i,
            total_chunks: 10,
            data,
            checksum,
        };
        
        p2p_service.receive_chunk(chunk).unwrap();
    }
    
    let partial_transfer = p2p_service.get_transfer(&transfer_id).unwrap();
    assert_eq!(partial_transfer.status, TransferStatus::Transferring);
    assert!((partial_transfer.progress() - 0.5).abs() < 0.01); // ~50%
    
    let missing = partial_transfer.missing_chunks();
    assert_eq!(missing.len(), 5);
    assert_eq!(missing, vec![5, 6, 7, 8, 9]);
}

#[tokio::test]
async fn test_signaling_server() {
    let server = SignalingServer::new();
    
    let (tx1, mut rx1) = mpsc::unbounded_channel();
    let (tx2, mut rx2) = mpsc::unbounded_channel();
    
    // Register two peers
    server.register_session(1, "session_1".to_string(), tx1);
    server.register_session(2, "session_2".to_string(), tx2);
    
    assert!(server.is_online(1));
    assert!(server.is_online(2));
    assert_eq!(server.online_count(), 2);
    
    // Send offer from peer 1 to peer 2
    let offer = SignalingMessage::Offer {
        from: 1,
        to: 2,
        transfer_id: "transfer_123".to_string(),
        sdp: "sdp_offer_data".to_string(),
    };
    
    server.handle_message(offer).unwrap();
    
    // Peer 2 should receive the offer
    let received = rx2.recv().await.unwrap();
    match received {
        SignalingMessage::Offer { from, to, transfer_id, .. } => {
            assert_eq!(from, 1);
            assert_eq!(to, 2);
            assert_eq!(transfer_id, "transfer_123");
        }
        _ => panic!("Expected Offer message"),
    }
    
    // Send answer from peer 2 to peer 1
    let answer = SignalingMessage::Answer {
        from: 2,
        to: 1,
        transfer_id: "transfer_123".to_string(),
        sdp: "sdp_answer_data".to_string(),
    };
    
    server.handle_message(answer).unwrap();
    
    // Peer 1 should receive the answer
    let received = rx1.recv().await.unwrap();
    match received {
        SignalingMessage::Answer { from, to, .. } => {
            assert_eq!(from, 2);
            assert_eq!(to, 1);
        }
        _ => panic!("Expected Answer message"),
    }
}

#[tokio::test]
async fn test_file_storage_service() {
    let config = StorageConfig {
        storage_dir: "/tmp/clawmesh_p2p_test".to_string(),
        retention_days: 7,
        max_file_size: 10 * 1024 * 1024,
        max_storage_size: 100 * 1024 * 1024,
    };
    
    let storage = FileStorageService::new(config).await.unwrap();
    
    // Store a file
    let file_data = b"This is a test file for P2P transfer".to_vec();
    let stored = storage.store_file(
        "transfer_123".to_string(),
        1,
        2,
        "test_document.txt".to_string(),
        "text/plain".to_string(),
        file_data.clone(),
    ).await.unwrap();
    
    assert_eq!(stored.sender_id, 1);
    assert_eq!(stored.recipient_id, 2);
    assert_eq!(stored.file_size, file_data.len() as u64);
    assert!(!stored.downloaded);
    
    // Retrieve the file
    let retrieved = storage.retrieve_file(&stored.file_id).await.unwrap();
    assert_eq!(retrieved, file_data);
    
    // Check metadata updated
    let metadata = storage.get_file_metadata(&stored.file_id).unwrap();
    assert!(metadata.downloaded);
    assert_eq!(metadata.download_count, 1);
    
    // Get files for recipient
    let recipient_files = storage.get_files_for_recipient(2);
    assert_eq!(recipient_files.len(), 0); // Already downloaded
    
    // Cleanup
    storage.delete_file(&stored.file_id).await.unwrap();
}

#[tokio::test]
async fn test_end_to_end_p2p_workflow() {
    // Setup services
    let p2p_service = Arc::new(P2PTransferService::new(P2PConfig {
        chunk_size: 1024,
        ..Default::default()
    }));
    
    let storage_config = StorageConfig {
        storage_dir: "/tmp/clawmesh_e2e_test".to_string(),
        ..Default::default()
    };
    let storage = Arc::new(FileStorageService::new(storage_config).await.unwrap());
    
    // Scenario: Sender online, recipient offline
    p2p_service.register_peer(PeerConnection {
        user_id: 1,
        session_id: "session_1".to_string(),
        connection_id: "conn_1".to_string(),
        ip_address: "192.168.1.100".to_string(),
        port: 8080,
        online: true,
        last_seen: chrono::Utc::now(),
    });
    
    // Initiate transfer
    let transfer = p2p_service.initiate_transfer(
        1,
        2,
        "important_file.pdf".to_string(),
        5000,
        "application/pdf".to_string(),
    ).unwrap();
    
    // Should use server relay
    assert_eq!(transfer.mode, TransferMode::ServerRelay);
    
    let transfer_id = transfer.transfer_id.clone();
    
    // Send file in chunks
    let file_data = vec![0xAB; 5000];
    let chunks: Vec<_> = file_data.chunks(1024)
        .enumerate()
        .map(|(i, chunk_data)| {
            let checksum = FileChunk::calculate_checksum(chunk_data);
            FileChunk {
                transfer_id: transfer_id.clone(),
                chunk_index: i as u32,
                total_chunks: 5,
                data: chunk_data.to_vec(),
                checksum,
            }
        })
        .collect();
    
    for chunk in chunks {
        p2p_service.receive_chunk(chunk).unwrap();
    }
    
    // Verify transfer completed
    let completed = p2p_service.get_transfer(&transfer_id).unwrap();
    assert_eq!(completed.status, TransferStatus::Completed);
    
    // Get file data and store for recipient
    let received_data = p2p_service.get_file_data(&transfer_id).unwrap();
    assert_eq!(received_data.len(), 5000);
    
    let stored = storage.store_file(
        transfer_id.clone(),
        1,
        2,
        "important_file.pdf".to_string(),
        "application/pdf".to_string(),
        received_data,
    ).await.unwrap();
    
    // Recipient comes online and retrieves file
    let retrieved = storage.retrieve_file(&stored.file_id).await.unwrap();
    assert_eq!(retrieved.len(), 5000);
    
    // Cleanup
    storage.delete_file(&stored.file_id).await.unwrap();
}

#[tokio::test]
async fn test_concurrent_transfers() {
    let p2p_service = Arc::new(P2PTransferService::new(P2PConfig {
        max_concurrent_transfers: 5,
        chunk_size: 1024,
        ..Default::default()
    }));
    
    // Initiate multiple transfers
    let mut transfers = Vec::new();
    for i in 0..3 {
        let transfer = p2p_service.initiate_transfer(
            1,
            i + 2,
            format!("file_{}.bin", i),
            1000,
            "application/octet-stream".to_string(),
        ).unwrap();
        transfers.push(transfer);
    }
    
    let stats = p2p_service.get_stats();
    assert_eq!(stats.total_transfers, 3);
    assert_eq!(stats.relay_transfers, 3);
}

#[tokio::test]
async fn test_transfer_cancellation() {
    let p2p_service = Arc::new(P2PTransferService::new(P2PConfig::default()));
    
    let transfer = p2p_service.initiate_transfer(
        1, 2, "file.bin".to_string(), 1000, "application/octet-stream".to_string()
    ).unwrap();
    
    let transfer_id = transfer.transfer_id.clone();
    
    // Cancel transfer
    p2p_service.cancel_transfer(&transfer_id).unwrap();
    
    let cancelled = p2p_service.get_transfer(&transfer_id).unwrap();
    assert_eq!(cancelled.status, TransferStatus::Cancelled);
}

#[tokio::test]
async fn test_p2p_retry_with_relay() {
    let p2p_service = Arc::new(P2PTransferService::new(P2PConfig::default()));
    
    // Register both peers
    p2p_service.register_peer(PeerConnection {
        user_id: 1,
        session_id: "s1".to_string(),
        connection_id: "c1".to_string(),
        ip_address: "192.168.1.100".to_string(),
        port: 8080,
        online: true,
        last_seen: chrono::Utc::now(),
    });
    
    p2p_service.register_peer(PeerConnection {
        user_id: 2,
        session_id: "s2".to_string(),
        connection_id: "c2".to_string(),
        ip_address: "192.168.1.101".to_string(),
        port: 8080,
        online: true,
        last_seen: chrono::Utc::now(),
    });
    
    let transfer = p2p_service.initiate_transfer(
        1, 2, "file.bin".to_string(), 1000, "application/octet-stream".to_string()
    ).unwrap();
    
    assert_eq!(transfer.mode, TransferMode::P2P);
    
    // P2P fails, retry with relay
    p2p_service.retry_with_relay(&transfer.transfer_id).unwrap();
    
    let retried = p2p_service.get_transfer(&transfer.transfer_id).unwrap();
    assert_eq!(retried.mode, TransferMode::ServerRelay);
}
