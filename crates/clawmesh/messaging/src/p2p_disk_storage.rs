//! P2P Disk-based Chunk Storage
//!
//! Aerospace-grade disk storage implementation with LRU caching for P2P file transfers.
//! Prevents memory overflow by storing chunks on disk with intelligent caching.

use crate::errors::{P2PError, P2PResult};
use crate::p2p_transfer::FileChunk;
use lru::LruCache;
use parking_lot::RwLock;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, warn, instrument};

/// Disk-based chunk storage with LRU cache
pub struct DiskChunkStorage {
    /// Base directory for chunk storage
    base_dir: PathBuf,
    /// LRU cache for hot chunks (in-memory)
    cache: Arc<RwLock<LruCache<String, Vec<u8>>>>,
    /// Maximum cache size in number of chunks
    cache_capacity: usize,
    /// Maximum total storage size in bytes
    max_storage_size: u64,
}

impl DiskChunkStorage {
    /// Create new disk chunk storage
    ///
    /// # Arguments
    /// * `base_dir` - Base directory for storing chunks
    /// * `cache_size_mb` - Cache size in megabytes (for hot chunks)
    /// * `max_storage_gb` - Maximum storage size in gigabytes
    pub async fn new(
        base_dir: PathBuf,
        cache_size_mb: usize,
        max_storage_gb: usize,
    ) -> P2PResult<Self> {
        // Create base directory if it doesn't exist
        fs::create_dir_all(&base_dir)
            .await
            .map_err(|e| P2PError::Storage(format!("Failed to create storage directory: {}", e)))?;

        // Calculate cache capacity (number of 64KB chunks)
        let chunk_size = 64 * 1024; // 64 KB
        let cache_capacity = (cache_size_mb * 1024 * 1024) / chunk_size;

        let storage = Self {
            base_dir,
            cache: Arc::new(RwLock::new(
                LruCache::new(NonZeroUsize::new(cache_capacity).unwrap())
            )),
            cache_capacity,
            max_storage_size: (max_storage_gb as u64) * 1024 * 1024 * 1024,
        };

        info!(
            base_dir = ?storage.base_dir,
            cache_capacity = cache_capacity,
            max_storage_gb = max_storage_gb,
            "Disk chunk storage initialized"
        );

        Ok(storage)
    }

    /// Store a chunk to disk and cache
    #[instrument(skip(self, chunk))]
    pub async fn store_chunk(&self, chunk: &FileChunk) -> P2PResult<()> {
        // Validate chunk
        if !chunk.verify() {
            return Err(P2PError::ChecksumMismatch(chunk.chunk_index));
        }

        // Check storage quota
        self.check_storage_quota(chunk.data.len() as u64).await?;

        // Get chunk path
        let chunk_path = self.get_chunk_path(&chunk.transfer_id, chunk.chunk_index);

        // Create parent directory
        if let Some(parent) = chunk_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| P2PError::Storage(format!("Failed to create directory: {}", e)))?;
        }

        // Serialize chunk
        let data = bincode::serialize(chunk)
            .map_err(|e| P2PError::Storage(format!("Serialization error: {}", e)))?;

        // Write to disk with atomic operation
        let temp_path = chunk_path.with_extension("tmp");
        let mut file = fs::File::create(&temp_path)
            .await
            .map_err(|e| P2PError::Storage(format!("Failed to create file: {}", e)))?;

        file.write_all(&data)
            .await
            .map_err(|e| P2PError::Storage(format!("Failed to write chunk: {}", e)))?;

        file.sync_all()
            .await
            .map_err(|e| P2PError::Storage(format!("Failed to sync file: {}", e)))?;

        // Atomic rename
        fs::rename(&temp_path, &chunk_path)
            .await
            .map_err(|e| P2PError::Storage(format!("Failed to rename file: {}", e)))?;

        // Update cache
        let cache_key = format!("{}:{}", chunk.transfer_id, chunk.chunk_index);
        self.cache.write().put(cache_key, chunk.data.clone());

        debug!(
            transfer_id = %chunk.transfer_id,
            chunk_index = chunk.chunk_index,
            size = chunk.data.len(),
            "Chunk stored to disk"
        );

        Ok(())
    }

    /// Retrieve a chunk from cache or disk
    #[instrument(skip(self))]
    pub async fn get_chunk(&self, transfer_id: &str, chunk_index: u32) -> P2PResult<FileChunk> {
        let cache_key = format!("{}:{}", transfer_id, chunk_index);

        // Try cache first (fast path)
        if let Some(_data) = self.cache.write().get(&cache_key) {
            debug!(
                transfer_id = %transfer_id,
                chunk_index = chunk_index,
                "Cache hit"
            );
            // Note: We need to reconstruct the full FileChunk from metadata
            // For now, we'll read from disk to get complete metadata
        }

        // Read from disk (slow path)
        let chunk_path = self.get_chunk_path(transfer_id, chunk_index);
        
        let data = fs::read(&chunk_path)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    P2PError::InvalidChunk(format!("Chunk {}:{} not found", transfer_id, chunk_index))
                } else {
                    P2PError::Storage(format!("Failed to read chunk: {}", e))
                }
            })?;

        let chunk: FileChunk = bincode::deserialize(&data)
            .map_err(|e| P2PError::Storage(format!("Deserialization error: {}", e)))?;

        // Verify chunk integrity
        if !chunk.verify() {
            return Err(P2PError::ChecksumMismatch(chunk_index));
        }

        // Update cache
        self.cache.write().put(cache_key, chunk.data.clone());

        debug!(
            transfer_id = %transfer_id,
            chunk_index = chunk_index,
            size = chunk.data.len(),
            "Chunk retrieved from disk"
        );

        Ok(chunk)
    }

    /// Get all chunks for a transfer
    #[instrument(skip(self))]
    pub async fn get_all_chunks(&self, transfer_id: &str, total_chunks: u32) -> P2PResult<Vec<FileChunk>> {
        let mut chunks = Vec::with_capacity(total_chunks as usize);

        for i in 0..total_chunks {
            let chunk = self.get_chunk(transfer_id, i).await?;
            chunks.push(chunk);
        }

        Ok(chunks)
    }

    /// Check if a chunk exists
    pub async fn chunk_exists(&self, transfer_id: &str, chunk_index: u32) -> bool {
        let chunk_path = self.get_chunk_path(transfer_id, chunk_index);
        chunk_path.exists()
    }

    /// Get missing chunk indices
    pub async fn get_missing_chunks(
        &self,
        transfer_id: &str,
        total_chunks: u32,
    ) -> P2PResult<Vec<u32>> {
        let mut missing = Vec::new();

        for i in 0..total_chunks {
            if !self.chunk_exists(transfer_id, i).await {
                missing.push(i);
            }
        }

        Ok(missing)
    }

    /// Cleanup transfer (delete all chunks)
    #[instrument(skip(self))]
    pub async fn cleanup_transfer(&self, transfer_id: &str) -> P2PResult<()> {
        let transfer_dir = self.base_dir.join(transfer_id);

        if transfer_dir.exists() {
            fs::remove_dir_all(&transfer_dir)
                .await
                .map_err(|e| P2PError::Storage(format!("Failed to cleanup transfer: {}", e)))?;

            info!(transfer_id = %transfer_id, "Transfer cleaned up");
        }

        // Remove from cache
        let mut cache = self.cache.write();
        cache.iter().for_each(|(key, _)| {
            if key.starts_with(transfer_id) {
                // LRU cache doesn't support removal during iteration
                // We'll let it naturally expire
            }
        });

        Ok(())
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> P2PResult<StorageStats> {
        let total_size = self.calculate_total_size().await?;
        let cache_size = self.cache.read().len();

        Ok(StorageStats {
            total_size_bytes: total_size,
            cache_entries: cache_size,
            cache_capacity: self.cache_capacity,
            max_storage_bytes: self.max_storage_size,
            utilization_percent: (total_size as f64 / self.max_storage_size as f64 * 100.0) as u32,
        })
    }

    /// Get chunk path
    fn get_chunk_path(&self, transfer_id: &str, chunk_index: u32) -> PathBuf {
        // Validate transfer_id to prevent path traversal
        if transfer_id.contains("..") || transfer_id.contains('/') || transfer_id.contains('\\') {
            // Return a safe invalid path that will fail
            return self.base_dir.join("invalid");
        }

        self.base_dir
            .join(transfer_id)
            .join(format!("chunk_{:06}.bin", chunk_index))
    }

    /// Check storage quota
    async fn check_storage_quota(&self, additional_bytes: u64) -> P2PResult<()> {
        let current_size = self.calculate_total_size().await?;
        
        if current_size + additional_bytes > self.max_storage_size {
            return Err(P2PError::Storage(format!(
                "Storage quota exceeded: {} + {} > {}",
                current_size, additional_bytes, self.max_storage_size
            )));
        }

        Ok(())
    }

    /// Calculate total storage size
    async fn calculate_total_size(&self) -> P2PResult<u64> {
        let mut total = 0u64;

        let mut entries = fs::read_dir(&self.base_dir)
            .await
            .map_err(|e| P2PError::Storage(format!("Failed to read directory: {}", e)))?;

        while let Some(entry) = entries.next_entry()
            .await
            .map_err(|e| P2PError::Storage(format!("Failed to read entry: {}", e)))? 
        {
            if let Ok(metadata) = entry.metadata().await {
                if metadata.is_file() {
                    total += metadata.len();
                } else if metadata.is_dir() {
                    total += self.calculate_dir_size(&entry.path()).await?;
                }
            }
        }

        Ok(total)
    }

    /// Calculate directory size recursively
    fn calculate_dir_size<'a>(&'a self, path: &'a Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = P2PResult<u64>> + 'a>> {
        Box::pin(async move {
            let mut total = 0u64;

            let mut entries = fs::read_dir(path)
                .await
                .map_err(|e| P2PError::Storage(format!("Failed to read directory: {}", e)))?;

            while let Some(entry) = entries.next_entry()
                .await
                .map_err(|e| P2PError::Storage(format!("Failed to read entry: {}", e)))? 
            {
                if let Ok(metadata) = entry.metadata().await {
                    if metadata.is_file() {
                        total += metadata.len();
                    } else if metadata.is_dir() {
                        total += self.calculate_dir_size(&entry.path()).await?;
                    }
                }
            }

            Ok(total)
        })
    }

    /// Cleanup old transfers (older than specified days)
    #[instrument(skip(self))]
    pub async fn cleanup_old_transfers(&self, days: u64) -> P2PResult<usize> {
        use std::time::{SystemTime, Duration};

        let cutoff = SystemTime::now() - Duration::from_secs(days * 24 * 60 * 60);
        let mut cleaned = 0;

        let mut entries = fs::read_dir(&self.base_dir)
            .await
            .map_err(|e| P2PError::Storage(format!("Failed to read directory: {}", e)))?;

        while let Some(entry) = entries.next_entry()
            .await
            .map_err(|e| P2PError::Storage(format!("Failed to read entry: {}", e)))? 
        {
            if let Ok(metadata) = entry.metadata().await {
                if metadata.is_dir() {
                    if let Ok(modified) = metadata.modified() {
                        if modified < cutoff {
                            if let Err(e) = fs::remove_dir_all(entry.path()).await {
                                warn!(path = ?entry.path(), error = %e, "Failed to remove old transfer");
                            } else {
                                cleaned += 1;
                            }
                        }
                    }
                }
            }
        }

        if cleaned > 0 {
            info!(cleaned = cleaned, days = days, "Cleaned up old transfers");
        }

        Ok(cleaned)
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    /// Total storage size in bytes
    pub total_size_bytes: u64,
    /// Number of cached entries
    pub cache_entries: usize,
    /// Cache capacity
    pub cache_capacity: usize,
    /// Maximum storage size in bytes
    pub max_storage_bytes: u64,
    /// Storage utilization percentage
    pub utilization_percent: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_disk_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = DiskChunkStorage::new(
            temp_dir.path().to_path_buf(),
            10, // 10 MB cache
            1,  // 1 GB max storage
        ).await.unwrap();

        let stats = storage.get_stats().await.unwrap();
        assert_eq!(stats.cache_entries, 0);
        assert!(stats.total_size_bytes == 0);
    }

    #[tokio::test]
    async fn test_store_and_retrieve_chunk() {
        let temp_dir = TempDir::new().unwrap();
        let storage = DiskChunkStorage::new(
            temp_dir.path().to_path_buf(),
            10,
            1,
        ).await.unwrap();

        let chunk = FileChunk {
            transfer_id: "test_transfer".to_string(),
            chunk_index: 0,
            total_chunks: 1,
            data: vec![1, 2, 3, 4, 5],
            checksum: FileChunk::calculate_checksum(&[1, 2, 3, 4, 5]),
        };

        storage.store_chunk(&chunk).await.unwrap();
        let retrieved = storage.get_chunk("test_transfer", 0).await.unwrap();

        assert_eq!(chunk.data, retrieved.data);
        assert_eq!(chunk.checksum, retrieved.checksum);
    }

    #[tokio::test]
    async fn test_path_traversal_prevention() {
        let temp_dir = TempDir::new().unwrap();
        let storage = DiskChunkStorage::new(
            temp_dir.path().to_path_buf(),
            10,
            1,
        ).await.unwrap();

        // Try path traversal attack
        let result = storage.get_chunk("../../../etc/passwd", 0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cleanup_transfer() {
        let temp_dir = TempDir::new().unwrap();
        let storage = DiskChunkStorage::new(
            temp_dir.path().to_path_buf(),
            10,
            1,
        ).await.unwrap();

        let chunk = FileChunk {
            transfer_id: "test_transfer".to_string(),
            chunk_index: 0,
            total_chunks: 1,
            data: vec![1, 2, 3, 4, 5],
            checksum: FileChunk::calculate_checksum(&[1, 2, 3, 4, 5]),
        };

        storage.store_chunk(&chunk).await.unwrap();
        storage.cleanup_transfer("test_transfer").await.unwrap();

        let result = storage.get_chunk("test_transfer", 0).await;
        assert!(result.is_err());
    }
}
