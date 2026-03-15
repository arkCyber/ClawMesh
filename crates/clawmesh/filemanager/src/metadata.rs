//! File metadata management

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// File type enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileType {
    /// Image file
    Image,
    /// Video file
    Video,
    /// Audio file
    Audio,
    /// Document file
    Document,
    /// Archive file
    Archive,
    /// Other file type
    Other,
}

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// File ID
    pub file_id: String,
    /// Original filename
    pub filename: String,
    /// File type
    pub file_type: FileType,
    /// MIME type
    pub mime_type: String,
    /// File size in bytes
    pub size: u64,
    /// SHA256 hash
    pub hash: String,
    /// Uploader user ID
    pub uploader_id: i32,
    /// Upload timestamp
    pub uploaded_at: DateTime<Utc>,
    /// Last accessed timestamp
    pub last_accessed_at: Option<DateTime<Utc>>,
    /// Download count
    pub download_count: i32,
    /// Is file deleted (soft delete)
    pub is_deleted: bool,
    /// Additional metadata (width, height, duration, etc.)
    pub extra: serde_json::Value,
}

impl FileMetadata {
    /// Create new file metadata
    #[must_use]
    pub fn new(
        file_id: String,
        filename: String,
        mime_type: String,
        size: u64,
        hash: String,
        uploader_id: i32,
    ) -> Self {
        let file_type = Self::detect_file_type(&mime_type);
        
        Self {
            file_id,
            filename,
            file_type,
            mime_type,
            size,
            hash,
            uploader_id,
            uploaded_at: Utc::now(),
            last_accessed_at: None,
            download_count: 0,
            is_deleted: false,
            extra: serde_json::Value::Null,
        }
    }

    /// Detect file type from MIME type
    fn detect_file_type(mime_type: &str) -> FileType {
        if mime_type.starts_with("image/") {
            FileType::Image
        } else if mime_type.starts_with("video/") {
            FileType::Video
        } else if mime_type.starts_with("audio/") {
            FileType::Audio
        } else if mime_type.starts_with("application/pdf") 
            || mime_type.starts_with("application/msword")
            || mime_type.starts_with("application/vnd.") {
            FileType::Document
        } else if mime_type.starts_with("application/zip")
            || mime_type.starts_with("application/x-tar")
            || mime_type.starts_with("application/x-rar") {
            FileType::Archive
        } else {
            FileType::Other
        }
    }

    /// Record file access
    pub fn record_access(&mut self) {
        self.last_accessed_at = Some(Utc::now());
        self.download_count += 1;
    }

    /// Mark file as deleted (soft delete)
    pub fn mark_deleted(&mut self) {
        self.is_deleted = true;
    }

    /// Check if file is image
    #[must_use]
    pub fn is_image(&self) -> bool {
        self.file_type == FileType::Image
    }

    /// Check if file is video
    #[must_use]
    pub fn is_video(&self) -> bool {
        self.file_type == FileType::Video
    }

    /// Get human-readable file size
    #[must_use]
    pub fn human_readable_size(&self) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = self.size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_file_type() {
        assert_eq!(
            FileMetadata::detect_file_type("image/jpeg"),
            FileType::Image
        );
        assert_eq!(
            FileMetadata::detect_file_type("video/mp4"),
            FileType::Video
        );
        assert_eq!(
            FileMetadata::detect_file_type("application/pdf"),
            FileType::Document
        );
    }

    #[test]
    fn test_file_metadata_creation() {
        let metadata = FileMetadata::new(
            "abc123".to_string(),
            "test.jpg".to_string(),
            "image/jpeg".to_string(),
            1024,
            "hash123".to_string(),
            1,
        );

        assert_eq!(metadata.file_type, FileType::Image);
        assert!(metadata.is_image());
        assert!(!metadata.is_video());
        assert_eq!(metadata.download_count, 0);
    }

    #[test]
    fn test_record_access() {
        let mut metadata = FileMetadata::new(
            "abc123".to_string(),
            "test.jpg".to_string(),
            "image/jpeg".to_string(),
            1024,
            "hash123".to_string(),
            1,
        );

        metadata.record_access();
        assert_eq!(metadata.download_count, 1);
        assert!(metadata.last_accessed_at.is_some());
    }

    #[test]
    fn test_human_readable_size() {
        let metadata = FileMetadata::new(
            "abc123".to_string(),
            "test.jpg".to_string(),
            "image/jpeg".to_string(),
            1536,
            "hash123".to_string(),
            1,
        );

        let size_str = metadata.human_readable_size();
        assert!(size_str.contains("KB"));
    }

    #[test]
    fn test_mark_deleted() {
        let mut metadata = FileMetadata::new(
            "abc123".to_string(),
            "test.jpg".to_string(),
            "image/jpeg".to_string(),
            1024,
            "hash123".to_string(),
            1,
        );

        assert!(!metadata.is_deleted);
        metadata.mark_deleted();
        assert!(metadata.is_deleted);
    }
}
