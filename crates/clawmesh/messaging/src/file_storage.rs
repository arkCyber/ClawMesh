//! File Storage Module
//!
//! Handles server-side file storage for relay transfers and offline recipients.
//! Implements temporary storage with automatic cleanup.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, warn, error, instrument};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

/// Stored file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredFile {
    /// File ID
    pub file_id: String,
    /// Transfer ID
    pub transfer_id: String,
    /// Sender user ID
    pub sender_id: i32,
    /// Recipient user ID
    pub recipient_id: i32,
    /// Original file name
    pub file_name: String,
    /// File size (bytes)
    pub file_size: u64,
    /// MIME type
    pub mime_type: String,
    /// Storage path
    pub storage_path: String,
    /// Stored at
    pub stored_at: DateTime<Utc>,
    /// Expires at
    pub expires_at: DateTime<Utc>,
    /// Downloaded
    pub downloaded: bool,
    /// Download count
    pub download_count: u32,
}

/// File storage service
pub struct FileStorageService {
    /// Storage directory
    storage_dir: PathBuf,
    /// File metadata
    files: Arc<RwLock<HashMap<String, StoredFile>>>,
    /// Configuration
    config: StorageConfig,
}

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Storage directory path
    pub storage_dir: String,
    /// File retention days
    pub retention_days: i64,
    /// Max file size (bytes)
    pub max_file_size: u64,
    /// Max storage size (bytes)
    pub max_storage_size: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            storage_dir: "/tmp/clawmesh/files".to_string(),
            retention_days: 7,
            max_file_size: 100 * 1024 * 1024,   // 100 MB
            max_storage_size: 10 * 1024 * 1024 * 1024, // 10 GB
        }
    }
}

impl FileStorageService {
    /// Create new file storage service
    pub async fn new(config: StorageConfig) -> Result<Self, String> {
        let storage_dir = PathBuf::from(&config.storage_dir);
        
        // Create storage directory if it doesn't exist
        fs::create_dir_all(&storage_dir)
            .await
            .map_err(|e| format!("Failed to create storage directory: {}", e))?;

        Ok(Self {
            storage_dir,
            files: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }

    /// Store file
    #[instrument(skip(self, data))]
    pub async fn store_file(
        &self,
        transfer_id: String,
        sender_id: i32,
        recipient_id: i32,
        file_name: String,
        mime_type: String,
        data: Vec<u8>,
    ) -> Result<StoredFile, String> {
        let file_size = data.len() as u64;

        // Validate file size
        if file_size > self.config.max_file_size {
            return Err(format!(
                "File size {} exceeds maximum {}",
                file_size, self.config.max_file_size
            ));
        }

        // Check total storage size
        let current_size = self.get_total_storage_size();
        if current_size + file_size > self.config.max_storage_size {
            return Err("Storage quota exceeded".to_string());
        }

        // Generate file ID and path
        let file_id = Uuid::new_v4().to_string();
        let storage_path = self.storage_dir.join(&file_id);

        // Write file to disk
        let mut file = fs::File::create(&storage_path)
            .await
            .map_err(|e| format!("Failed to create file: {}", e))?;

        file.write_all(&data)
            .await
            .map_err(|e| format!("Failed to write file: {}", e))?;

        file.sync_all()
            .await
            .map_err(|e| format!("Failed to sync file: {}", e))?;

        // Create metadata
        let stored_file = StoredFile {
            file_id: file_id.clone(),
            transfer_id,
            sender_id,
            recipient_id,
            file_name,
            file_size,
            mime_type,
            storage_path: storage_path.to_string_lossy().to_string(),
            stored_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(self.config.retention_days),
            downloaded: false,
            download_count: 0,
        };

        self.files.write().insert(file_id.clone(), stored_file.clone());

        info!(
            file_id = %file_id,
            file_size = file_size,
            recipient_id = recipient_id,
            "File stored"
        );

        Ok(stored_file)
    }

    /// Retrieve file
    #[instrument(skip(self))]
    pub async fn retrieve_file(&self, file_id: &str) -> Result<Vec<u8>, String> {
        let mut files = self.files.write();
        let file = files.get_mut(file_id)
            .ok_or("File not found")?;

        // Check if expired
        if Utc::now() > file.expires_at {
            return Err("File has expired".to_string());
        }

        // Read file from disk
        let data = fs::read(&file.storage_path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Update metadata
        file.downloaded = true;
        file.download_count += 1;

        debug!(
            file_id = %file_id,
            download_count = file.download_count,
            "File retrieved"
        );

        Ok(data)
    }

    /// Get file metadata
    pub fn get_file_metadata(&self, file_id: &str) -> Option<StoredFile> {
        self.files.read().get(file_id).cloned()
    }

    /// Delete file
    #[instrument(skip(self))]
    pub async fn delete_file(&self, file_id: &str) -> Result<(), String> {
        let mut files = self.files.write();
        let file = files.remove(file_id)
            .ok_or("File not found")?;

        // Delete from disk
        fs::remove_file(&file.storage_path)
            .await
            .map_err(|e| format!("Failed to delete file: {}", e))?;

        info!(file_id = %file_id, "File deleted");
        Ok(())
    }

    /// Get files for recipient
    pub fn get_files_for_recipient(&self, recipient_id: i32) -> Vec<StoredFile> {
        self.files.read()
            .values()
            .filter(|f| f.recipient_id == recipient_id && !f.downloaded)
            .cloned()
            .collect()
    }

    /// Clean up expired files
    #[instrument(skip(self))]
    pub async fn cleanup_expired_files(&self) -> Result<usize, String> {
        let now = Utc::now();
        let files = self.files.read();
        
        let expired_files: Vec<String> = files.iter()
            .filter(|(_, f)| now > f.expires_at)
            .map(|(id, _)| id.clone())
            .collect();

        drop(files);

        let count = expired_files.len();
        for file_id in expired_files {
            if let Err(e) = self.delete_file(&file_id).await {
                warn!(file_id = %file_id, error = %e, "Failed to delete expired file");
            }
        }

        if count > 0 {
            info!(count = count, "Cleaned up expired files");
        }

        Ok(count)
    }

    /// Get total storage size
    fn get_total_storage_size(&self) -> u64 {
        self.files.read()
            .values()
            .map(|f| f.file_size)
            .sum()
    }

    /// Get storage statistics
    pub fn get_stats(&self) -> StorageStats {
        let files = self.files.read();
        
        let total_files = files.len();
        let total_size = files.values().map(|f| f.file_size).sum();
        let downloaded_files = files.values().filter(|f| f.downloaded).count();
        let pending_files = files.values().filter(|f| !f.downloaded).count();

        StorageStats {
            total_files,
            total_size,
            downloaded_files,
            pending_files,
            max_storage_size: self.config.max_storage_size,
        }
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_files: usize,
    pub total_size: u64,
    pub downloaded_files: usize,
    pub pending_files: usize,
    pub max_storage_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_retrieve_file() {
        let config = StorageConfig {
            storage_dir: "/tmp/clawmesh_test".to_string(),
            retention_days: 7,
            max_file_size: 10 * 1024 * 1024,
            max_storage_size: 100 * 1024 * 1024,
        };

        let service = FileStorageService::new(config).await.unwrap();

        let data = b"Hello, World!".to_vec();
        let stored = service.store_file(
            "transfer_123".to_string(),
            1,
            2,
            "test.txt".to_string(),
            "text/plain".to_string(),
            data.clone(),
        ).await.unwrap();

        assert_eq!(stored.file_size, 13);
        assert_eq!(stored.sender_id, 1);
        assert_eq!(stored.recipient_id, 2);

        let retrieved = service.retrieve_file(&stored.file_id).await.unwrap();
        assert_eq!(retrieved, data);

        // Cleanup
        service.delete_file(&stored.file_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_file_size_limit() {
        let config = StorageConfig {
            storage_dir: "/tmp/clawmesh_test".to_string(),
            retention_days: 7,
            max_file_size: 100, // 100 bytes limit
            max_storage_size: 1000,
        };

        let service = FileStorageService::new(config).await.unwrap();

        let data = vec![0u8; 200]; // 200 bytes
        let result = service.store_file(
            "transfer_123".to_string(),
            1,
            2,
            "large.bin".to_string(),
            "application/octet-stream".to_string(),
            data,
        ).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_files_for_recipient() {
        let config = StorageConfig {
            storage_dir: "/tmp/clawmesh_test".to_string(),
            ..Default::default()
        };

        let service = FileStorageService::new(config).await.unwrap();

        // Store files for different recipients
        service.store_file(
            "t1".to_string(), 1, 2, "file1.txt".to_string(), "text/plain".to_string(), b"data1".to_vec()
        ).await.unwrap();

        service.store_file(
            "t2".to_string(), 1, 2, "file2.txt".to_string(), "text/plain".to_string(), b"data2".to_vec()
        ).await.unwrap();

        service.store_file(
            "t3".to_string(), 1, 3, "file3.txt".to_string(), "text/plain".to_string(), b"data3".to_vec()
        ).await.unwrap();

        let files_for_2 = service.get_files_for_recipient(2);
        assert_eq!(files_for_2.len(), 2);

        let files_for_3 = service.get_files_for_recipient(3);
        assert_eq!(files_for_3.len(), 1);
    }

    #[tokio::test]
    async fn test_storage_stats() {
        let config = StorageConfig {
            storage_dir: "/tmp/clawmesh_test".to_string(),
            ..Default::default()
        };

        let service = FileStorageService::new(config).await.unwrap();

        service.store_file(
            "t1".to_string(), 1, 2, "file1.txt".to_string(), "text/plain".to_string(), vec![0u8; 100]
        ).await.unwrap();

        let stats = service.get_stats();
        assert_eq!(stats.total_files, 1);
        assert_eq!(stats.total_size, 100);
        assert_eq!(stats.pending_files, 1);
    }
}
