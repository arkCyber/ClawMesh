//! ClawMesh File Management System
//! 
//! Provides file upload, storage, organization, and multimedia support

pub mod storage;
pub mod upload;
pub mod metadata;
pub mod thumbnail;

pub use storage::{FileStorage, StorageBackend};
pub use upload::{FileUpload, UploadOptions};
pub use metadata::{FileMetadata, FileType};
pub use thumbnail::ThumbnailGenerator;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// File manager configuration
#[derive(Debug, Clone)]
pub struct FileManagerConfig {
    /// Maximum file size in bytes
    pub max_file_size: u64,
    /// Allowed file types
    pub allowed_types: Vec<String>,
    /// Storage path
    pub storage_path: String,
    /// Enable thumbnail generation
    pub enable_thumbnails: bool,
    /// Thumbnail size
    pub thumbnail_size: (u32, u32),
}

impl Default for FileManagerConfig {
    fn default() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100 MB
            allowed_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "image/webp".to_string(),
                "video/mp4".to_string(),
                "video/webm".to_string(),
                "application/pdf".to_string(),
            ],
            storage_path: "./uploads".to_string(),
            enable_thumbnails: true,
            thumbnail_size: (300, 300),
        }
    }
}

/// File upload result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    /// File ID
    pub file_id: String,
    /// Original filename
    pub filename: String,
    /// File URL
    pub url: String,
    /// Thumbnail URL (if generated)
    pub thumbnail_url: Option<String>,
    /// File size in bytes
    pub size: u64,
    /// MIME type
    pub mime_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FileManagerConfig::default();
        assert_eq!(config.max_file_size, 100 * 1024 * 1024);
        assert!(config.enable_thumbnails);
    }
}
