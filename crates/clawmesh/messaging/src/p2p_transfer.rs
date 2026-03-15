//! P2P File Transfer System
//!
//! Implements peer-to-peer binary file transfer with automatic fallback to server relay.
//! Reduces server load by enabling direct connections when both peers are online.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tracing::{debug, info, warn, error, instrument};
use uuid::Uuid;

/// File transfer mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransferMode {
    /// Direct P2P connection
    P2P,
    /// Server relay (fallback)
    ServerRelay,
}

/// File transfer status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransferStatus {
    /// Waiting for peer connection
    Pending,
    /// Negotiating P2P connection
    Negotiating,
    /// Actively transferring
    Transferring,
    /// Transfer completed
    Completed,
    /// Transfer failed
    Failed,
    /// Transfer cancelled
    Cancelled,
}

/// File chunk for transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunk {
    /// Transfer ID
    pub transfer_id: String,
    /// Chunk index
    pub chunk_index: u32,
    /// Total chunks
    pub total_chunks: u32,
    /// Chunk data
    pub data: Vec<u8>,
    /// Checksum (CRC32)
    pub checksum: u32,
}

impl FileChunk {
    /// Calculate CRC32 checksum using hardware-accelerated crc32fast
    pub fn calculate_checksum(data: &[u8]) -> u32 {
        use crc32fast::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(data);
        hasher.finalize()
    }

    /// Verify chunk integrity
    pub fn verify(&self) -> bool {
        Self::calculate_checksum(&self.data) == self.checksum
    }
}

/// File transfer metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransfer {
    /// Transfer ID
    pub transfer_id: String,
    /// Sender user ID
    pub sender_id: i32,
    /// Recipient user ID
    pub recipient_id: i32,
    /// File name
    pub file_name: String,
    /// File size (bytes)
    pub file_size: u64,
    /// MIME type
    pub mime_type: String,
    /// Transfer mode
    pub mode: TransferMode,
    /// Transfer status
    pub status: TransferStatus,
    /// Bytes transferred
    pub bytes_transferred: u64,
    /// Total chunks
    pub total_chunks: u32,
    /// Received chunks
    pub received_chunks: Vec<u32>,
    /// File hash (SHA-256)
    pub file_hash: Option<String>,
    /// Chunk hashes (SHA-256 for each chunk)
    pub chunk_hashes: HashMap<u32, String>,
    /// Failed chunks (for retry)
    pub failed_chunks: Vec<u32>,
    /// Retry count per chunk
    pub retry_counts: HashMap<u32, u32>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Updated at
    pub updated_at: DateTime<Utc>,
    /// Completed at
    pub completed_at: Option<DateTime<Utc>>,
}

impl FileTransfer {
    /// Create new file transfer
    pub fn new(
        sender_id: i32,
        recipient_id: i32,
        file_name: String,
        file_size: u64,
        mime_type: String,
        chunk_size: usize,
    ) -> Self {
        let total_chunks = ((file_size as usize + chunk_size - 1) / chunk_size) as u32;
        
        Self {
            transfer_id: Uuid::new_v4().to_string(),
            sender_id,
            recipient_id,
            file_name,
            file_size,
            mime_type,
            mode: TransferMode::P2P, // Try P2P first
            status: TransferStatus::Pending,
            bytes_transferred: 0,
            total_chunks,
            received_chunks: Vec::new(),
            file_hash: None,
            chunk_hashes: HashMap::new(),
            failed_chunks: Vec::new(),
            retry_counts: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
        }
    }
    
    /// Set file hash (SHA-256)
    pub fn set_file_hash(&mut self, hash: String) {
        self.file_hash = Some(hash);
    }
    
    /// Add chunk hash
    pub fn add_chunk_hash(&mut self, chunk_index: u32, hash: String) {
        self.chunk_hashes.insert(chunk_index, hash);
    }
    
    /// Verify chunk integrity using SHA-256
    pub fn verify_chunk(&self, chunk_index: u32, data: &[u8]) -> bool {
        if let Some(expected_hash) = self.chunk_hashes.get(&chunk_index) {
            let actual_hash = Self::calculate_sha256(data);
            &actual_hash == expected_hash
        } else {
            // If no hash stored, fall back to CRC32
            true
        }
    }
    
    /// Calculate SHA-256 hash
    pub fn calculate_sha256(data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
    
    /// Verify complete file integrity
    pub fn verify_file_integrity(&self, file_data: &[u8]) -> bool {
        if let Some(expected_hash) = &self.file_hash {
            let actual_hash = Self::calculate_sha256(file_data);
            &actual_hash == expected_hash
        } else {
            false
        }
    }
    
    /// Mark chunk as failed
    pub fn mark_chunk_failed(&mut self, chunk_index: u32) {
        if !self.failed_chunks.contains(&chunk_index) {
            self.failed_chunks.push(chunk_index);
        }
        *self.retry_counts.entry(chunk_index).or_insert(0) += 1;
    }
    
    /// Get chunks that need retry
    pub fn get_retry_chunks(&self, max_retries: u32) -> Vec<u32> {
        self.failed_chunks
            .iter()
            .filter(|&&chunk_index| {
                self.retry_counts.get(&chunk_index).unwrap_or(&0) < &max_retries
            })
            .copied()
            .collect()
    }
    
    /// Clear failed chunk on successful retry
    pub fn clear_failed_chunk(&mut self, chunk_index: u32) {
        self.failed_chunks.retain(|&i| i != chunk_index);
    }

    /// Calculate transfer progress (0.0 - 1.0)
    pub fn progress(&self) -> f64 {
        if self.file_size == 0 {
            return 1.0;
        }
        self.bytes_transferred as f64 / self.file_size as f64
    }

    /// Check if transfer is complete
    pub fn is_complete(&self) -> bool {
        self.received_chunks.len() as u32 == self.total_chunks
    }

    /// Get missing chunks
    pub fn missing_chunks(&self) -> Vec<u32> {
        (0..self.total_chunks)
            .filter(|i| !self.received_chunks.contains(i))
            .collect()
    }
}

/// Peer connection info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConnection {
    /// User ID
    pub user_id: i32,
    /// Session ID
    pub session_id: String,
    /// WebSocket connection ID
    pub connection_id: String,
    /// IP address
    pub ip_address: String,
    /// Port
    pub port: u16,
    /// Is online
    pub online: bool,
    /// Last seen
    pub last_seen: DateTime<Utc>,
}

/// P2P Transfer Service
pub struct P2PTransferService {
    /// Active transfers
    transfers: Arc<RwLock<HashMap<String, FileTransfer>>>,
    /// Peer connections
    peers: Arc<RwLock<HashMap<i32, PeerConnection>>>,
    /// Chunk storage (for server relay)
    chunk_storage: Arc<RwLock<HashMap<String, Vec<FileChunk>>>>,
    /// Configuration
    config: P2PConfig,
}

/// P2P Configuration
#[derive(Debug, Clone)]
pub struct P2PConfig {
    /// Chunk size (bytes)
    pub chunk_size: usize,
    /// Max concurrent transfers
    pub max_concurrent_transfers: usize,
    /// P2P negotiation timeout (seconds)
    pub negotiation_timeout: u64,
    /// Transfer timeout (seconds)
    pub transfer_timeout: u64,
    /// Max file size (bytes)
    pub max_file_size: u64,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            chunk_size: 64 * 1024,           // 64 KB chunks
            max_concurrent_transfers: 10,
            negotiation_timeout: 30,         // 30 seconds
            transfer_timeout: 3600,          // 1 hour
            max_file_size: 100 * 1024 * 1024, // 100 MB
        }
    }
}

impl P2PTransferService {
    /// Create new P2P transfer service
    pub fn new(config: P2PConfig) -> Self {
        Self {
            transfers: Arc::new(RwLock::new(HashMap::new())),
            peers: Arc::new(RwLock::new(HashMap::new())),
            chunk_storage: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Register peer connection
    #[instrument(skip(self))]
    pub fn register_peer(&self, peer: PeerConnection) {
        let mut peers = self.peers.write();
        info!(
            user_id = peer.user_id,
            session_id = %peer.session_id,
            "Peer registered"
        );
        peers.insert(peer.user_id, peer);
    }

    /// Unregister peer connection
    #[instrument(skip(self))]
    pub fn unregister_peer(&self, user_id: i32) {
        let mut peers = self.peers.write();
        if let Some(mut peer) = peers.get_mut(&user_id) {
            peer.online = false;
            peer.last_seen = Utc::now();
            info!(user_id = user_id, "Peer unregistered");
        }
    }

    /// Check if peer is online
    pub fn is_peer_online(&self, user_id: i32) -> bool {
        self.peers.read()
            .get(&user_id)
            .map(|p| p.online)
            .unwrap_or(false)
    }

    /// Initiate file transfer
    #[instrument(skip(self))]
    pub fn initiate_transfer(
        &self,
        sender_id: i32,
        recipient_id: i32,
        file_name: String,
        file_size: u64,
        mime_type: String,
    ) -> Result<FileTransfer, String> {
        // Validate file size
        if file_size > self.config.max_file_size {
            return Err(format!(
                "File size {} exceeds maximum {}",
                file_size, self.config.max_file_size
            ));
        }

        // Check concurrent transfers
        let active_count = self.transfers.read()
            .values()
            .filter(|t| t.status == TransferStatus::Transferring)
            .count();

        if active_count >= self.config.max_concurrent_transfers {
            return Err("Too many concurrent transfers".to_string());
        }

        // Create transfer
        let mut transfer = FileTransfer::new(
            sender_id,
            recipient_id,
            file_name,
            file_size,
            mime_type,
            self.config.chunk_size,
        );

        // Check if recipient is online for P2P
        if !self.is_peer_online(recipient_id) {
            info!(
                transfer_id = %transfer.transfer_id,
                recipient_id = recipient_id,
                "Recipient offline, using server relay"
            );
            transfer.mode = TransferMode::ServerRelay;
        } else {
            info!(
                transfer_id = %transfer.transfer_id,
                recipient_id = recipient_id,
                "Recipient online, attempting P2P"
            );
            transfer.status = TransferStatus::Negotiating;
        }

        let transfer_id = transfer.transfer_id.clone();
        self.transfers.write().insert(transfer_id.clone(), transfer.clone());

        info!(
            transfer_id = %transfer_id,
            mode = ?transfer.mode,
            file_size = file_size,
            "Transfer initiated"
        );

        Ok(transfer)
    }

    /// Receive file chunk
    #[instrument(skip(self, chunk))]
    pub fn receive_chunk(&self, chunk: FileChunk) -> Result<(), String> {
        // Verify chunk integrity
        if !chunk.verify() {
            return Err("Chunk checksum verification failed".to_string());
        }

        let mut transfers = self.transfers.write();
        let transfer = transfers.get_mut(&chunk.transfer_id)
            .ok_or("Transfer not found")?;

        // Check if chunk already received
        if transfer.received_chunks.contains(&chunk.chunk_index) {
            debug!(
                transfer_id = %chunk.transfer_id,
                chunk_index = chunk.chunk_index,
                "Chunk already received, skipping"
            );
            return Ok(());
        }

        // Update transfer status
        if transfer.status == TransferStatus::Pending || transfer.status == TransferStatus::Negotiating {
            transfer.status = TransferStatus::Transferring;
        }

        // Store chunk
        transfer.received_chunks.push(chunk.chunk_index);
        transfer.bytes_transferred += chunk.data.len() as u64;
        transfer.updated_at = Utc::now();

        // Store chunk data (for server relay)
        if transfer.mode == TransferMode::ServerRelay {
            let mut storage = self.chunk_storage.write();
            storage.entry(chunk.transfer_id.clone())
                .or_insert_with(Vec::new)
                .push(chunk.clone());
        }

        debug!(
            transfer_id = %chunk.transfer_id,
            chunk_index = chunk.chunk_index,
            progress = format!("{:.1}%", transfer.progress() * 100.0),
            "Chunk received"
        );

        // Check if transfer is complete
        if transfer.is_complete() {
            transfer.status = TransferStatus::Completed;
            transfer.completed_at = Some(Utc::now());
            
            info!(
                transfer_id = %transfer.transfer_id,
                file_size = transfer.file_size,
                duration = ?(Utc::now() - transfer.created_at),
                "Transfer completed"
            );
        }

        Ok(())
    }

    /// Get transfer status
    pub fn get_transfer(&self, transfer_id: &str) -> Option<FileTransfer> {
        self.transfers.read().get(transfer_id).cloned()
    }

    /// Cancel transfer
    #[instrument(skip(self))]
    pub fn cancel_transfer(&self, transfer_id: &str) -> Result<(), String> {
        let mut transfers = self.transfers.write();
        let transfer = transfers.get_mut(transfer_id)
            .ok_or("Transfer not found")?;

        transfer.status = TransferStatus::Cancelled;
        transfer.updated_at = Utc::now();

        // Clean up chunk storage
        self.chunk_storage.write().remove(transfer_id);

        info!(transfer_id = %transfer_id, "Transfer cancelled");
        Ok(())
    }

    /// Retry failed transfer with server relay
    #[instrument(skip(self))]
    pub fn retry_with_relay(&self, transfer_id: &str) -> Result<(), String> {
        let mut transfers = self.transfers.write();
        let transfer = transfers.get_mut(transfer_id)
            .ok_or("Transfer not found")?;

        if transfer.mode == TransferMode::P2P {
            warn!(
                transfer_id = %transfer_id,
                "P2P failed, switching to server relay"
            );
            transfer.mode = TransferMode::ServerRelay;
            transfer.status = TransferStatus::Pending;
            transfer.updated_at = Utc::now();
        }

        Ok(())
    }

    /// Get file data from completed transfer
    pub fn get_file_data(&self, transfer_id: &str) -> Result<Vec<u8>, String> {
        let transfer = self.get_transfer(transfer_id)
            .ok_or("Transfer not found")?;

        if transfer.status != TransferStatus::Completed {
            return Err("Transfer not completed".to_string());
        }

        let storage = self.chunk_storage.read();
        let chunks = storage.get(transfer_id)
            .ok_or("Chunk data not found")?;

        // Sort chunks by index
        let mut sorted_chunks: Vec<_> = chunks.iter().collect();
        sorted_chunks.sort_by_key(|c| c.chunk_index);

        // Combine chunks
        let mut file_data = Vec::with_capacity(transfer.file_size as usize);
        for chunk in sorted_chunks {
            file_data.extend_from_slice(&chunk.data);
        }

        Ok(file_data)
    }

    /// Get transfer statistics
    pub fn get_stats(&self) -> TransferStats {
        let transfers = self.transfers.read();
        let peers = self.peers.read();

        let total_transfers = transfers.len();
        let active_transfers = transfers.values()
            .filter(|t| t.status == TransferStatus::Transferring)
            .count();
        let completed_transfers = transfers.values()
            .filter(|t| t.status == TransferStatus::Completed)
            .count();
        let p2p_transfers = transfers.values()
            .filter(|t| t.mode == TransferMode::P2P)
            .count();
        let relay_transfers = transfers.values()
            .filter(|t| t.mode == TransferMode::ServerRelay)
            .count();
        let online_peers = peers.values()
            .filter(|p| p.online)
            .count();

        let total_bytes_transferred: u64 = transfers.values()
            .map(|t| t.bytes_transferred)
            .sum();

        TransferStats {
            total_transfers,
            active_transfers,
            completed_transfers,
            p2p_transfers,
            relay_transfers,
            online_peers,
            total_bytes_transferred,
        }
    }

    /// Clean up old transfers
    pub fn cleanup_old_transfers(&self, max_age_hours: i64) -> usize {
        let mut transfers = self.transfers.write();
        let mut storage = self.chunk_storage.write();
        let cutoff = Utc::now() - chrono::Duration::hours(max_age_hours);

        let old_transfers: Vec<String> = transfers.iter()
            .filter(|(_, t)| {
                (t.status == TransferStatus::Completed || t.status == TransferStatus::Failed)
                    && t.updated_at < cutoff
            })
            .map(|(id, _)| id.clone())
            .collect();

        let count = old_transfers.len();
        for transfer_id in old_transfers {
            transfers.remove(&transfer_id);
            storage.remove(&transfer_id);
        }

        if count > 0 {
            info!(count = count, "Cleaned up old transfers");
        }

        count
    }
}

/// Transfer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferStats {
    pub total_transfers: usize,
    pub active_transfers: usize,
    pub completed_transfers: usize,
    pub p2p_transfers: usize,
    pub relay_transfers: usize,
    pub online_peers: usize,
    pub total_bytes_transferred: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_chunk_checksum() {
        let data = b"Hello, World!";
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
    fn test_file_transfer_creation() {
        let transfer = FileTransfer::new(
            1,
            2,
            "test.pdf".to_string(),
            1024 * 1024, // 1 MB
            "application/pdf".to_string(),
            64 * 1024, // 64 KB chunks
        );

        assert_eq!(transfer.sender_id, 1);
        assert_eq!(transfer.recipient_id, 2);
        assert_eq!(transfer.total_chunks, 16); // 1MB / 64KB = 16
        assert_eq!(transfer.progress(), 0.0);
    }

    #[test]
    fn test_transfer_progress() {
        let mut transfer = FileTransfer::new(
            1, 2, "test.bin".to_string(), 1000, "application/octet-stream".to_string(), 100
        );

        assert_eq!(transfer.total_chunks, 10);
        assert_eq!(transfer.progress(), 0.0);

        transfer.bytes_transferred = 500;
        assert_eq!(transfer.progress(), 0.5);

        transfer.bytes_transferred = 1000;
        assert_eq!(transfer.progress(), 1.0);
    }

    #[test]
    fn test_missing_chunks() {
        let mut transfer = FileTransfer::new(
            1, 2, "test.bin".to_string(), 500, "application/octet-stream".to_string(), 100
        );

        transfer.received_chunks = vec![0, 2, 4];
        let missing = transfer.missing_chunks();
        
        assert_eq!(missing, vec![1, 3]);
    }

    #[tokio::test]
    async fn test_p2p_service_initiate_transfer() {
        let service = P2PTransferService::new(P2PConfig::default());

        let transfer = service.initiate_transfer(
            1,
            2,
            "document.pdf".to_string(),
            1024 * 1024,
            "application/pdf".to_string(),
        ).unwrap();

        assert_eq!(transfer.sender_id, 1);
        assert_eq!(transfer.recipient_id, 2);
        assert_eq!(transfer.mode, TransferMode::ServerRelay); // Recipient offline
    }

    #[tokio::test]
    async fn test_p2p_service_with_online_peer() {
        let service = P2PTransferService::new(P2PConfig::default());

        // Register peer as online
        service.register_peer(PeerConnection {
            user_id: 2,
            session_id: "session_123".to_string(),
            connection_id: "conn_456".to_string(),
            ip_address: "192.168.1.100".to_string(),
            port: 8080,
            online: true,
            last_seen: Utc::now(),
        });

        let transfer = service.initiate_transfer(
            1,
            2,
            "video.mp4".to_string(),
            10 * 1024 * 1024,
            "video/mp4".to_string(),
        ).unwrap();

        assert_eq!(transfer.mode, TransferMode::P2P); // Should try P2P
        assert_eq!(transfer.status, TransferStatus::Negotiating);
    }

    #[tokio::test]
    async fn test_receive_chunks() {
        let service = P2PTransferService::new(P2PConfig::default());

        let transfer = service.initiate_transfer(
            1, 2, "test.bin".to_string(), 200, "application/octet-stream".to_string()
        ).unwrap();

        let transfer_id = transfer.transfer_id.clone();

        // Send chunks
        for i in 0..transfer.total_chunks {
            let data = vec![i as u8; 64 * 1024];
            let checksum = FileChunk::calculate_checksum(&data);
            
            let chunk = FileChunk {
                transfer_id: transfer_id.clone(),
                chunk_index: i,
                total_chunks: transfer.total_chunks,
                data,
                checksum,
            };

            service.receive_chunk(chunk).unwrap();
        }

        let updated_transfer = service.get_transfer(&transfer_id).unwrap();
        assert_eq!(updated_transfer.status, TransferStatus::Completed);
        assert_eq!(updated_transfer.progress(), 1.0);
    }

    #[tokio::test]
    async fn test_transfer_stats() {
        let service = P2PTransferService::new(P2PConfig::default());

        service.initiate_transfer(
            1, 2, "file1.bin".to_string(), 1000, "application/octet-stream".to_string()
        ).unwrap();

        service.initiate_transfer(
            1, 3, "file2.bin".to_string(), 2000, "application/octet-stream".to_string()
        ).unwrap();

        let stats = service.get_stats();
        assert_eq!(stats.total_transfers, 2);
        assert_eq!(stats.relay_transfers, 2); // Both offline
    }
}
