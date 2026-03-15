//! File upload handling

use crate::{FileManagerConfig, UploadResult};
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::Path;

/// File upload handler
pub struct FileUpload {
    config: FileManagerConfig,
}

/// Upload options
#[derive(Debug, Clone)]
pub struct UploadOptions {
    /// User ID uploading the file
    pub user_id: i32,
    /// Original filename
    pub filename: String,
    /// MIME type
    pub mime_type: String,
    /// Generate thumbnail
    pub generate_thumbnail: bool,
}

impl FileUpload {
    /// Create a new file upload handler
    #[must_use]
    pub fn new(config: FileManagerConfig) -> Self {
        Self { config }
    }

    /// Upload a file
    ///
    /// # Errors
    /// Returns error if upload fails
    pub async fn upload(&self, data: &[u8], options: UploadOptions) -> Result<UploadResult> {
        // Validate file size
        if data.len() as u64 > self.config.max_file_size {
            return Err(anyhow::anyhow!(
                "File size {} exceeds maximum {}",
                data.len(),
                self.config.max_file_size
            ));
        }

        // Validate MIME type
        if !self.config.allowed_types.contains(&options.mime_type) {
            return Err(anyhow::anyhow!(
                "File type {} not allowed",
                options.mime_type
            ));
        }

        // Generate file ID from hash
        let file_id = self.generate_file_id(data);

        // TODO: Store file using storage backend
        // TODO: Generate thumbnail if requested
        // TODO: Store metadata in database

        Ok(UploadResult {
            file_id: file_id.clone(),
            filename: options.filename,
            url: format!("/files/{}", file_id),
            thumbnail_url: None,
            size: data.len() as u64,
            mime_type: options.mime_type,
        })
    }

    /// Generate file ID from content hash
    fn generate_file_id(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        format!("{:x}", hash)
    }

    /// Validate filename
    ///
    /// # Errors
    /// Returns error if filename is invalid
    pub fn validate_filename(&self, filename: &str) -> Result<()> {
        if filename.is_empty() {
            return Err(anyhow::anyhow!("Filename cannot be empty"));
        }

        if filename.len() > 255 {
            return Err(anyhow::anyhow!("Filename too long (max 255 characters)"));
        }

        // Check for path traversal attempts
        let path = Path::new(filename);
        if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err(anyhow::anyhow!("Invalid filename: path traversal detected"));
        }

        Ok(())
    }

    /// Get file extension from filename
    #[must_use]
    pub fn get_extension(&self, filename: &str) -> Option<String> {
        Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }

    /// Detect MIME type from file extension
    #[must_use]
    pub fn detect_mime_type(&self, filename: &str) -> Option<String> {
        self.get_extension(filename).and_then(|ext| {
            match ext.as_str() {
                "jpg" | "jpeg" => Some("image/jpeg".to_string()),
                "png" => Some("image/png".to_string()),
                "gif" => Some("image/gif".to_string()),
                "webp" => Some("image/webp".to_string()),
                "mp4" => Some("video/mp4".to_string()),
                "webm" => Some("video/webm".to_string()),
                "pdf" => Some("application/pdf".to_string()),
                _ => None,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_file_id() {
        let config = FileManagerConfig::default();
        let upload = FileUpload::new(config);
        
        let data = b"test data";
        let id1 = upload.generate_file_id(data);
        let id2 = upload.generate_file_id(data);
        
        assert_eq!(id1, id2); // Same data should produce same ID
        assert_eq!(id1.len(), 64); // SHA256 hash is 64 hex chars
    }

    #[test]
    fn test_validate_filename() {
        let config = FileManagerConfig::default();
        let upload = FileUpload::new(config);
        
        assert!(upload.validate_filename("test.jpg").is_ok());
        assert!(upload.validate_filename("").is_err());
        assert!(upload.validate_filename("../etc/passwd").is_err());
        assert!(upload.validate_filename(&"a".repeat(300)).is_err());
    }

    #[test]
    fn test_get_extension() {
        let config = FileManagerConfig::default();
        let upload = FileUpload::new(config);
        
        assert_eq!(upload.get_extension("test.jpg"), Some("jpg".to_string()));
        assert_eq!(upload.get_extension("test.PNG"), Some("png".to_string()));
        assert_eq!(upload.get_extension("noext"), None);
    }

    #[test]
    fn test_detect_mime_type() {
        let config = FileManagerConfig::default();
        let upload = FileUpload::new(config);
        
        assert_eq!(upload.detect_mime_type("test.jpg"), Some("image/jpeg".to_string()));
        assert_eq!(upload.detect_mime_type("test.png"), Some("image/png".to_string()));
        assert_eq!(upload.detect_mime_type("test.unknown"), None);
    }

    #[tokio::test]
    async fn test_upload_size_validation() {
        let mut config = FileManagerConfig::default();
        config.max_file_size = 100;
        let upload = FileUpload::new(config);
        
        let data = vec![0u8; 200];
        let options = UploadOptions {
            user_id: 1,
            filename: "test.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            generate_thumbnail: false,
        };
        
        let result = upload.upload(&data, options).await;
        assert!(result.is_err());
    }
}
