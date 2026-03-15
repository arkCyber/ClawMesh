//! Aerospace-Grade Tests for P2P Transfer Integrity
//!
//! Test Coverage:
//! - SHA-256 hash calculation
//! - Chunk integrity verification
//! - File integrity verification
//! - Retry mechanism
//! - Failed chunk tracking

#[cfg(test)]
mod tests {
    use clawmesh_messaging::p2p_transfer::*;
    use std::collections::HashMap;
    
    // ============================================================================
    // Hash Calculation Tests
    // ============================================================================
    
    #[test]
    fn test_sha256_calculation() {
        let data = b"Hello, World!";
        let hash = FileTransfer::calculate_sha256(data);
        
        // Known SHA-256 hash for "Hello, World!"
        assert_eq!(
            hash,
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
        );
    }
    
    #[test]
    fn test_sha256_empty_data() {
        let data = b"";
        let hash = FileTransfer::calculate_sha256(data);
        
        // Known SHA-256 hash for empty string
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
    
    #[test]
    fn test_sha256_large_data() {
        let data = vec![0u8; 1024 * 1024]; // 1MB of zeros
        let hash = FileTransfer::calculate_sha256(&data);
        
        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex characters
    }
    
    #[test]
    fn test_sha256_deterministic() {
        let data = b"Test data";
        let hash1 = FileTransfer::calculate_sha256(data);
        let hash2 = FileTransfer::calculate_sha256(data);
        
        assert_eq!(hash1, hash2);
    }
    
    // ============================================================================
    // Chunk Verification Tests
    // ============================================================================
    
    #[test]
    fn test_chunk_verification_success() {
        let mut transfer = FileTransfer::new(
            1, 2, "test.txt".to_string(), 1000, "text/plain".to_string(), 100
        );
        
        let chunk_data = b"Test chunk data";
        let chunk_hash = FileTransfer::calculate_sha256(chunk_data);
        
        transfer.add_chunk_hash(0, chunk_hash);
        
        assert!(transfer.verify_chunk(0, chunk_data));
    }
    
    #[test]
    fn test_chunk_verification_failure() {
        let mut transfer = FileTransfer::new(
            1, 2, "test.txt".to_string(), 1000, "text/plain".to_string(), 100
        );
        
        let original_data = b"Original data";
        let corrupted_data = b"Corrupted data";
        
        let chunk_hash = FileTransfer::calculate_sha256(original_data);
        transfer.add_chunk_hash(0, chunk_hash);
        
        assert!(!transfer.verify_chunk(0, corrupted_data));
    }
    
    #[test]
    fn test_chunk_verification_no_hash() {
        let transfer = FileTransfer::new(
            1, 2, "test.txt".to_string(), 1000, "text/plain".to_string(), 100
        );
        
        let chunk_data = b"Any data";
        
        // Should return true when no hash is stored (fallback)
        assert!(transfer.verify_chunk(0, chunk_data));
    }
    
    // ============================================================================
    // File Integrity Tests
    // ============================================================================
    
    #[test]
    fn test_file_integrity_verification_success() {
        let mut transfer = FileTransfer::new(
            1, 2, "test.txt".to_string(), 1000, "text/plain".to_string(), 100
        );
        
        let file_data = b"Complete file data";
        let file_hash = FileTransfer::calculate_sha256(file_data);
        
        transfer.set_file_hash(file_hash);
        
        assert!(transfer.verify_file_integrity(file_data));
    }
    
    #[test]
    fn test_file_integrity_verification_failure() {
        let mut transfer = FileTransfer::new(
            1, 2, "test.txt".to_string(), 1000, "text/plain".to_string(), 100
        );
        
        let original_data = b"Original file";
        let corrupted_data = b"Corrupted file";
        
        let file_hash = FileTransfer::calculate_sha256(original_data);
        transfer.set_file_hash(file_hash);
        
        assert!(!transfer.verify_file_integrity(corrupted_data));
    }
    
    #[test]
    fn test_file_integrity_no_hash() {
        let transfer = FileTransfer::new(
            1, 2, "test.txt".to_string(), 1000, "text/plain".to_string(), 100
        );
        
        let file_data = b"Any data";
        
        // Should return false when no hash is set
        assert!(!transfer.verify_file_integrity(file_data));
    }
    
    // ============================================================================
    // Retry Mechanism Tests
    // ============================================================================
    
    #[test]
    fn test_mark_chunk_failed() {
        let mut transfer = FileTransfer::new(
            1, 2, "test.txt".to_string(), 1000, "text/plain".to_string(), 100
        );
        
        transfer.mark_chunk_failed(5);
        
        assert!(transfer.failed_chunks.contains(&5));
        assert_eq!(transfer.retry_counts.get(&5), Some(&1));
    }
    
    #[test]
    fn test_mark_chunk_failed_multiple_times() {
        let mut transfer = FileTransfer::new(
            1, 2, "test.txt".to_string(), 1000, "text/plain".to_string(), 100
        );
        
        transfer.mark_chunk_failed(5);
        transfer.mark_chunk_failed(5);
        transfer.mark_chunk_failed(5);
        
        assert_eq!(transfer.failed_chunks.len(), 1);
        assert_eq!(transfer.retry_counts.get(&5), Some(&3));
    }
    
    #[test]
    fn test_get_retry_chunks() {
        let mut transfer = FileTransfer::new(
            1, 2, "test.txt".to_string(), 1000, "text/plain".to_string(), 100
        );
        
        transfer.mark_chunk_failed(1);
        transfer.mark_chunk_failed(2);
        transfer.mark_chunk_failed(3);
        transfer.mark_chunk_failed(3); // Retry twice
        transfer.mark_chunk_failed(3); // Retry three times
        
        let retry_chunks = transfer.get_retry_chunks(3);
        
        // Chunks 1 and 2 should be retried (1 attempt each)
        // Chunk 3 should not be retried (3 attempts, at max)
        assert_eq!(retry_chunks.len(), 2);
        assert!(retry_chunks.contains(&1));
        assert!(retry_chunks.contains(&2));
        assert!(!retry_chunks.contains(&3));
    }
    
    #[test]
    fn test_clear_failed_chunk() {
        let mut transfer = FileTransfer::new(
            1, 2, "test.txt".to_string(), 1000, "text/plain".to_string(), 100
        );
        
        transfer.mark_chunk_failed(5);
        assert!(transfer.failed_chunks.contains(&5));
        
        transfer.clear_failed_chunk(5);
        assert!(!transfer.failed_chunks.contains(&5));
    }
    
    // ============================================================================
    // CRC32 Chunk Tests
    // ============================================================================
    
    #[test]
    fn test_file_chunk_crc32_calculation() {
        let data = b"Test data";
        let checksum = FileChunk::calculate_checksum(data);
        
        assert!(checksum > 0);
    }
    
    #[test]
    fn test_file_chunk_verify_success() {
        let data = b"Test chunk";
        let checksum = FileChunk::calculate_checksum(data);
        
        let chunk = FileChunk {
            transfer_id: "test".to_string(),
            chunk_index: 0,
            total_chunks: 1,
            data: data.to_vec(),
            checksum,
        };
        
        assert!(chunk.verify());
    }
    
    #[test]
    fn test_file_chunk_verify_failure() {
        let chunk = FileChunk {
            transfer_id: "test".to_string(),
            chunk_index: 0,
            total_chunks: 1,
            data: b"Test chunk".to_vec(),
            checksum: 12345, // Wrong checksum
        };
        
        assert!(!chunk.verify());
    }
    
    // ============================================================================
    // Performance Tests
    // ============================================================================
    
    #[test]
    fn test_sha256_performance() {
        use std::time::Instant;
        
        let data = vec![0u8; 1024]; // 1KB
        let start = Instant::now();
        
        for _ in 0..1000 {
            let _ = FileTransfer::calculate_sha256(&data);
        }
        
        let duration = start.elapsed();
        
        // Should hash 1000 x 1KB in less than 100ms
        assert!(duration.as_millis() < 100,
            "SHA-256 hashing took too long: {:?}", duration);
    }
    
    #[test]
    fn test_chunk_verification_performance() {
        use std::time::Instant;
        
        let mut transfer = FileTransfer::new(
            1, 2, "test.txt".to_string(), 1000, "text/plain".to_string(), 100
        );
        
        let data = b"Test data";
        let hash = FileTransfer::calculate_sha256(data);
        transfer.add_chunk_hash(0, hash);
        
        let start = Instant::now();
        
        for _ in 0..10000 {
            let _ = transfer.verify_chunk(0, data);
        }
        
        let duration = start.elapsed();
        
        // Should verify 10,000 chunks in less than 200ms
        assert!(duration.as_millis() < 200,
            "Chunk verification took too long: {:?}", duration);
    }
    
    // ============================================================================
    // Edge Case Tests
    // ============================================================================
    
    #[test]
    fn test_multiple_chunk_hashes() {
        let mut transfer = FileTransfer::new(
            1, 2, "test.txt".to_string(), 1000, "text/plain".to_string(), 100
        );
        
        for i in 0..10 {
            let data = format!("Chunk {}", i);
            let hash = FileTransfer::calculate_sha256(data.as_bytes());
            transfer.add_chunk_hash(i, hash);
        }
        
        assert_eq!(transfer.chunk_hashes.len(), 10);
    }
    
    #[test]
    fn test_retry_chunks_empty() {
        let transfer = FileTransfer::new(
            1, 2, "test.txt".to_string(), 1000, "text/plain".to_string(), 100
        );
        
        let retry_chunks = transfer.get_retry_chunks(3);
        assert!(retry_chunks.is_empty());
    }
    
    #[test]
    fn test_failed_chunks_deduplication() {
        let mut transfer = FileTransfer::new(
            1, 2, "test.txt".to_string(), 1000, "text/plain".to_string(), 100
        );
        
        transfer.mark_chunk_failed(5);
        transfer.mark_chunk_failed(5);
        transfer.mark_chunk_failed(5);
        
        // Should only appear once in failed_chunks
        assert_eq!(transfer.failed_chunks.len(), 1);
    }
}
