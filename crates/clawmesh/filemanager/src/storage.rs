//! File storage backend implementation

use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Storage backend trait
pub trait StorageBackend: Send + Sync {
    /// Store a file
    ///
    /// # Errors
    /// Returns error if storage fails
    fn store(&self, file_id: &str, data: &[u8]) -> impl std::future::Future<Output = Result<String>> + Send;

    /// Retrieve a file
    ///
    /// # Errors
    /// Returns error if retrieval fails
    fn retrieve(&self, file_id: &str) -> impl std::future::Future<Output = Result<Vec<u8>>> + Send;

    /// Delete a file
    ///
    /// # Errors
    /// Returns error if deletion fails
    fn delete(&self, file_id: &str) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Check if file exists
    ///
    /// # Errors
    /// Returns error if check fails
    fn exists(&self, file_id: &str) -> impl std::future::Future<Output = Result<bool>> + Send;
}

/// Local filesystem storage backend
pub struct FileStorage {
    base_path: PathBuf,
}

impl FileStorage {
    /// Create a new file storage
    ///
    /// # Errors
    /// Returns error if base path creation fails
    pub async fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        fs::create_dir_all(&base_path).await?;
        Ok(Self { base_path })
    }

    /// Get file path for a file ID
    fn get_file_path(&self, file_id: &str) -> PathBuf {
        // Use first 2 chars as subdirectory for better distribution
        let subdir = if file_id.len() >= 2 {
            &file_id[..2]
        } else {
            "00"
        };
        self.base_path.join(subdir).join(file_id)
    }

    /// Get file size
    ///
    /// # Errors
    /// Returns error if file doesn't exist or metadata retrieval fails
    pub async fn get_size(&self, file_id: &str) -> Result<u64> {
        let path = self.get_file_path(file_id);
        let metadata = fs::metadata(path).await?;
        Ok(metadata.len())
    }

    /// List all files
    ///
    /// # Errors
    /// Returns error if directory reading fails
    pub async fn list_files(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();
        let mut entries = fs::read_dir(&self.base_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                let mut subdir_entries = fs::read_dir(entry.path()).await?;
                while let Some(file_entry) = subdir_entries.next_entry().await? {
                    if let Some(filename) = file_entry.file_name().to_str() {
                        files.push(filename.to_string());
                    }
                }
            }
        }

        Ok(files)
    }
}

impl StorageBackend for FileStorage {
    async fn store(&self, file_id: &str, data: &[u8]) -> Result<String> {
        let path = self.get_file_path(file_id);
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write file
        let mut file = fs::File::create(&path).await?;
        file.write_all(data).await?;
        file.sync_all().await?;

        Ok(path.to_string_lossy().to_string())
    }

    async fn retrieve(&self, file_id: &str) -> Result<Vec<u8>> {
        let path = self.get_file_path(file_id);
        let data = fs::read(path).await?;
        Ok(data)
    }

    async fn delete(&self, file_id: &str) -> Result<()> {
        let path = self.get_file_path(file_id);
        fs::remove_file(path).await?;
        Ok(())
    }

    async fn exists(&self, file_id: &str) -> Result<bool> {
        let path = self.get_file_path(file_id);
        Ok(path.exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_storage() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let storage = FileStorage::new(temp_dir.path())
            .await
            .expect("Failed to create storage");

        let file_id = "test123";
        let data = b"Hello, World!";

        // Store file
        storage.store(file_id, data).await.expect("Failed to store");

        // Check exists
        assert!(storage.exists(file_id).await.expect("Failed to check exists"));

        // Retrieve file
        let retrieved = storage.retrieve(file_id).await.expect("Failed to retrieve");
        assert_eq!(retrieved, data);

        // Get size
        let size = storage.get_size(file_id).await.expect("Failed to get size");
        assert_eq!(size, data.len() as u64);

        // Delete file
        storage.delete(file_id).await.expect("Failed to delete");
        assert!(!storage.exists(file_id).await.expect("Failed to check exists"));
    }

    #[tokio::test]
    async fn test_list_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let storage = FileStorage::new(temp_dir.path())
            .await
            .expect("Failed to create storage");

        storage.store("file1", b"data1").await.expect("Failed to store");
        storage.store("file2", b"data2").await.expect("Failed to store");

        let files = storage.list_files().await.expect("Failed to list files");
        assert_eq!(files.len(), 2);
    }
}
