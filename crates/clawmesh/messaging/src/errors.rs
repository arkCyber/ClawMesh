//! Error Types for ClawMesh Messaging
//!
//! Aerospace-grade error handling with detailed error types and context.

use std::fmt;
use thiserror::Error;

/// Main error type for messaging operations
#[derive(Debug, Error)]
pub enum MessagingError {
    /// Database operation failed
    #[error("Database error: {0}")]
    Database(String),
    
    /// Redis operation failed
    #[error("Redis error: {0}")]
    Redis(String),
    
    /// Serialization/deserialization failed
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Encryption operation failed
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    /// Decryption operation failed
    #[error("Decryption error: {0}")]
    Decryption(String),
    
    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    /// Timeout
    #[error("Operation timeout: {0}")]
    Timeout(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// P2P transfer specific errors
#[derive(Debug, Error)]
pub enum P2PError {
    /// File too large
    #[error("File size {0} bytes exceeds maximum {1} bytes")]
    FileTooLarge(u64, u64),
    
    /// Transfer not found
    #[error("Transfer not found: {0}")]
    TransferNotFound(String),
    
    /// Peer not online
    #[error("Peer {0} is not online")]
    PeerOffline(i32),
    
    /// Checksum verification failed
    #[error("Checksum mismatch for chunk {0}")]
    ChecksumMismatch(u32),
    
    /// Invalid chunk
    #[error("Invalid chunk: {0}")]
    InvalidChunk(String),
    
    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),
    
    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    /// Transfer cancelled
    #[error("Transfer cancelled: {0}")]
    Cancelled(String),
    
    /// Concurrent transfer limit exceeded
    #[error("Concurrent transfer limit {0} exceeded")]
    TooManyTransfers(usize),
    
    /// Network error
    #[error("Network error: {0}")]
    Network(String),
}

/// File storage specific errors
#[derive(Debug, Error)]
pub enum StorageError {
    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    /// Invalid file ID
    #[error("Invalid file ID: {0}")]
    InvalidFileId(String),
    
    /// File size limit exceeded
    #[error("File size {0} bytes exceeds limit {1} bytes")]
    FileSizeLimitExceeded(u64, u64),
    
    /// Storage quota exceeded
    #[error("Storage quota exceeded: {0} bytes used, {1} bytes limit")]
    QuotaExceeded(u64, u64),
    
    /// Insufficient disk space
    #[error("Insufficient disk space: {0} bytes required, {1} bytes available")]
    InsufficientSpace(u64, u64),
    
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Path traversal attempt
    #[error("Path traversal attempt detected: {0}")]
    PathTraversal(String),
    
    /// File corrupted
    #[error("File corrupted: {0}")]
    Corrupted(String),
}

/// Encryption specific errors
#[derive(Debug, Error)]
pub enum EncryptionError {
    /// Key not found
    #[error("Encryption key not found: {0}")]
    KeyNotFound(String),
    
    /// Key expired
    #[error("Encryption key expired: {0}")]
    KeyExpired(String),
    
    /// Key revoked
    #[error("Encryption key revoked: {0}")]
    KeyRevoked(String),
    
    /// Invalid key
    #[error("Invalid encryption key: {0}")]
    InvalidKey(String),
    
    /// Encryption failed
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    /// Decryption failed
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    /// Key generation failed
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),
}

/// Result type alias for messaging operations
pub type Result<T> = std::result::Result<T, MessagingError>;

/// Result type alias for P2P operations
pub type P2PResult<T> = std::result::Result<T, P2PError>;

/// Result type alias for storage operations
pub type StorageResult<T> = std::result::Result<T, StorageError>;

/// Result type alias for encryption operations
pub type EncryptionResult<T> = std::result::Result<T, EncryptionError>;

// Conversion implementations
impl From<redis::RedisError> for MessagingError {
    fn from(err: redis::RedisError) -> Self {
        MessagingError::Redis(err.to_string())
    }
}

impl From<serde_json::Error> for MessagingError {
    fn from(err: serde_json::Error) -> Self {
        MessagingError::Serialization(err.to_string())
    }
}

impl From<P2PError> for MessagingError {
    fn from(err: P2PError) -> Self {
        MessagingError::Internal(err.to_string())
    }
}

impl From<StorageError> for MessagingError {
    fn from(err: StorageError) -> Self {
        MessagingError::Internal(err.to_string())
    }
}

impl From<EncryptionError> for MessagingError {
    fn from(err: EncryptionError) -> Self {
        MessagingError::Encryption(err.to_string())
    }
}
