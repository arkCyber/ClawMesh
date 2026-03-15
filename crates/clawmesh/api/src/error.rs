//! ClawMesh API Error Handling
//!
//! Provides comprehensive error types and handling for aerospace-grade reliability.
//! All errors are categorized, logged, and provide actionable information.

use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{error, warn};

/// Error codes for ClawMesh API
/// Following aerospace standards: unique, traceable, and categorized
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum ErrorCode {
    // Authentication errors (1000-1099)
    /// User not authenticated
    Unauthenticated = 1000,
    /// Invalid credentials
    InvalidCredentials = 1001,
    /// Session expired
    SessionExpired = 1002,
    /// Token invalid
    TokenInvalid = 1003,
    /// Insufficient permissions
    InsufficientPermissions = 1010,

    // Validation errors (2000-2099)
    /// Invalid input data
    InvalidInput = 2000,
    /// Missing required field
    MissingField = 2001,
    /// Field value out of range
    OutOfRange = 2002,
    /// Invalid format
    InvalidFormat = 2003,
    /// Content too long
    ContentTooLong = 2004,
    /// Content too short
    ContentTooShort = 2005,
    /// Invalid characters
    InvalidCharacters = 2006,

    // Resource errors (3000-3099)
    /// Resource not found
    NotFound = 3000,
    /// Resource already exists
    AlreadyExists = 3001,
    /// Resource deleted
    Deleted = 3002,
    /// Resource locked
    Locked = 3003,

    // Friendship errors (4000-4099)
    /// Already friends
    AlreadyFriends = 4000,
    /// Friend request already sent
    RequestAlreadySent = 4001,
    /// Friend request not found
    RequestNotFound = 4002,
    /// Cannot friend self
    CannotFriendSelf = 4003,
    /// User blocked
    UserBlocked = 4004,
    /// Blocked by user
    BlockedByUser = 4005,
    /// Max friends reached
    MaxFriendsReached = 4006,
    /// Max pending requests reached
    MaxPendingRequestsReached = 4007,

    // Messaging errors (5000-5099)
    /// Message not found
    MessageNotFound = 5000,
    /// Conversation not found
    ConversationNotFound = 5001,
    /// Cannot message self
    CannotMessageSelf = 5002,
    /// Message too long
    MessageTooLong = 5003,
    /// Rate limited
    RateLimited = 5010,

    // Credit errors (6000-6099)
    /// Insufficient credit
    InsufficientCredit = 6000,
    /// Credit operation failed
    CreditOperationFailed = 6001,

    // Agent errors (7000-7099)
    /// Agent not found
    AgentNotFound = 7000,
    /// Agent already exists
    AgentAlreadyExists = 7001,
    /// Invalid agent metadata
    InvalidAgentMetadata = 7002,
    /// Heartbeat timeout
    HeartbeatTimeout = 7003,

    // System errors (9000-9099)
    /// Internal server error
    InternalError = 9000,
    /// Database error
    DatabaseError = 9001,
    /// Service unavailable
    ServiceUnavailable = 9002,
    /// Configuration error
    ConfigurationError = 9003,
}

impl ErrorCode {
    /// Get the HTTP status code for this error
    #[must_use]
    pub const fn http_status(&self) -> StatusCode {
        match self {
            // Authentication errors -> 401/403
            Self::Unauthenticated | Self::InvalidCredentials | 
            Self::SessionExpired | Self::TokenInvalid => StatusCode::UNAUTHORIZED,
            Self::InsufficientPermissions => StatusCode::FORBIDDEN,

            // Validation errors -> 400
            Self::InvalidInput | Self::MissingField | Self::OutOfRange |
            Self::InvalidFormat | Self::ContentTooLong | Self::ContentTooShort |
            Self::InvalidCharacters => StatusCode::BAD_REQUEST,

            // Resource errors -> 404/409
            Self::NotFound | Self::Deleted => StatusCode::NOT_FOUND,
            Self::AlreadyExists | Self::Locked => StatusCode::CONFLICT,

            // Friendship errors -> 400/403/404/409
            Self::CannotFriendSelf | Self::MaxFriendsReached | 
            Self::MaxPendingRequestsReached => StatusCode::BAD_REQUEST,
            Self::UserBlocked | Self::BlockedByUser => StatusCode::FORBIDDEN,
            Self::RequestNotFound => StatusCode::NOT_FOUND,
            Self::AlreadyFriends | Self::RequestAlreadySent => StatusCode::CONFLICT,

            // Messaging errors -> 400/404/429
            Self::CannotMessageSelf | Self::MessageTooLong => StatusCode::BAD_REQUEST,
            Self::MessageNotFound | Self::ConversationNotFound => StatusCode::NOT_FOUND,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,

            // Credit errors -> 402/500
            Self::InsufficientCredit => StatusCode::PAYMENT_REQUIRED,
            Self::CreditOperationFailed => StatusCode::INTERNAL_SERVER_ERROR,

            // Agent errors -> 400/404
            Self::AgentNotFound => StatusCode::NOT_FOUND,
            Self::AgentAlreadyExists => StatusCode::CONFLICT,
            Self::InvalidAgentMetadata | Self::HeartbeatTimeout => StatusCode::BAD_REQUEST,

            // System errors -> 500/503
            Self::InternalError | Self::DatabaseError | 
            Self::ConfigurationError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    /// Get a human-readable description
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Unauthenticated => "Authentication required",
            Self::InvalidCredentials => "Invalid credentials provided",
            Self::SessionExpired => "Session has expired",
            Self::TokenInvalid => "Invalid authentication token",
            Self::InsufficientPermissions => "Insufficient permissions for this action",

            Self::InvalidInput => "Invalid input data",
            Self::MissingField => "Required field is missing",
            Self::OutOfRange => "Value is out of acceptable range",
            Self::InvalidFormat => "Invalid data format",
            Self::ContentTooLong => "Content exceeds maximum length",
            Self::ContentTooShort => "Content is below minimum length",
            Self::InvalidCharacters => "Content contains invalid characters",

            Self::NotFound => "Resource not found",
            Self::AlreadyExists => "Resource already exists",
            Self::Deleted => "Resource has been deleted",
            Self::Locked => "Resource is locked",

            Self::AlreadyFriends => "Already friends with this user",
            Self::RequestAlreadySent => "Friend request already sent",
            Self::RequestNotFound => "Friend request not found",
            Self::CannotFriendSelf => "Cannot send friend request to yourself",
            Self::UserBlocked => "User is blocked",
            Self::BlockedByUser => "You are blocked by this user",
            Self::MaxFriendsReached => "Maximum number of friends reached",
            Self::MaxPendingRequestsReached => "Maximum pending requests reached",

            Self::MessageNotFound => "Message not found",
            Self::ConversationNotFound => "Conversation not found",
            Self::CannotMessageSelf => "Cannot send message to yourself",
            Self::MessageTooLong => "Message exceeds maximum length",
            Self::RateLimited => "Rate limit exceeded, please try again later",

            Self::InsufficientCredit => "Insufficient credit for this action",
            Self::CreditOperationFailed => "Credit operation failed",

            Self::AgentNotFound => "Agent not found",
            Self::AgentAlreadyExists => "Agent already exists",
            Self::InvalidAgentMetadata => "Invalid agent metadata",
            Self::HeartbeatTimeout => "Agent heartbeat timeout",

            Self::InternalError => "Internal server error",
            Self::DatabaseError => "Database operation failed",
            Self::ServiceUnavailable => "Service temporarily unavailable",
            Self::ConfigurationError => "Configuration error",
        }
    }
}

/// ClawMesh API Error
/// Comprehensive error type with full context for debugging and user feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClawMeshError {
    /// Error code
    pub code: ErrorCode,
    /// Human-readable message
    pub message: String,
    /// Additional details (for debugging)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// Field name (for validation errors)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    /// Request ID for tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ClawMeshError {
    /// Create a new error
    #[must_use]
    pub fn new(code: ErrorCode) -> Self {
        Self {
            code,
            message: code.description().to_string(),
            details: None,
            field: None,
            request_id: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create an error with a custom message
    #[must_use]
    pub fn with_message(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
            field: None,
            request_id: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Add details to the error
    #[must_use]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Add field name (for validation errors)
    #[must_use]
    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    /// Add request ID for tracing
    #[must_use]
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Log the error with appropriate level
    pub fn log(&self) {
        let code = self.code as u32;
        if code >= 9000 {
            error!(
                code = code,
                message = %self.message,
                details = ?self.details,
                request_id = ?self.request_id,
                "System error occurred"
            );
        } else if code >= 4000 {
            warn!(
                code = code,
                message = %self.message,
                field = ?self.field,
                "Business logic error"
            );
        }
    }
}

impl fmt::Display for ClawMeshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code as u32, self.message)
    }
}

impl std::error::Error for ClawMeshError {}

impl ResponseError for ClawMeshError {
    fn status_code(&self) -> StatusCode {
        self.code.http_status()
    }

    fn error_response(&self) -> HttpResponse {
        self.log();
        HttpResponse::build(self.status_code()).json(self)
    }
}

/// Result type for ClawMesh API operations
pub type ClawMeshResult<T> = Result<T, ClawMeshError>;

/// Validation helper functions
pub mod validation {
    use super::*;

    /// Minimum username length
    pub const MIN_USERNAME_LENGTH: usize = 3;
    /// Maximum username length
    pub const MAX_USERNAME_LENGTH: usize = 50;
    /// Maximum message length
    pub const MAX_MESSAGE_LENGTH: usize = 10_000;
    /// Maximum nickname length
    pub const MAX_NICKNAME_LENGTH: usize = 50;
    /// Maximum notes length
    pub const MAX_NOTES_LENGTH: usize = 500;

    /// Validate that a string is not empty
    pub fn require_non_empty(value: &str, field: &str) -> ClawMeshResult<()> {
        if value.trim().is_empty() {
            return Err(ClawMeshError::with_message(
                ErrorCode::MissingField,
                format!("{} cannot be empty", field),
            ).with_field(field));
        }
        Ok(())
    }

    /// Validate string length
    pub fn validate_length(
        value: &str,
        field: &str,
        min: usize,
        max: usize,
    ) -> ClawMeshResult<()> {
        let len = value.chars().count();
        if len < min {
            return Err(ClawMeshError::with_message(
                ErrorCode::ContentTooShort,
                format!("{} must be at least {} characters", field, min),
            ).with_field(field));
        }
        if len > max {
            return Err(ClawMeshError::with_message(
                ErrorCode::ContentTooLong,
                format!("{} must be at most {} characters", field, max),
            ).with_field(field));
        }
        Ok(())
    }

    /// Validate user ID is positive
    pub fn validate_user_id(user_id: i32, field: &str) -> ClawMeshResult<()> {
        if user_id <= 0 {
            return Err(ClawMeshError::with_message(
                ErrorCode::InvalidInput,
                format!("{} must be a positive integer", field),
            ).with_field(field));
        }
        Ok(())
    }

    /// Validate that two user IDs are different (for self-actions)
    pub fn validate_not_self(user_id: i32, target_id: i32, action: &str) -> ClawMeshResult<()> {
        if user_id == target_id {
            return Err(ClawMeshError::with_message(
                ErrorCode::InvalidInput,
                format!("Cannot {} yourself", action),
            ));
        }
        Ok(())
    }

    /// Validate message content
    pub fn validate_message_content(content: &str) -> ClawMeshResult<()> {
        require_non_empty(content, "message")?;
        validate_length(content, "message", 1, MAX_MESSAGE_LENGTH)?;
        Ok(())
    }

    /// Validate username format
    pub fn validate_username(username: &str) -> ClawMeshResult<()> {
        require_non_empty(username, "username")?;
        validate_length(username, "username", MIN_USERNAME_LENGTH, MAX_USERNAME_LENGTH)?;
        
        // Must start with alphanumeric
        if let Some(first) = username.chars().next() {
            if !first.is_alphanumeric() {
                return Err(ClawMeshError::with_message(
                    ErrorCode::InvalidFormat,
                    "Username must start with a letter or number",
                ).with_field("username"));
            }
        }

        // Only allow alphanumeric, underscore, hyphen
        for c in username.chars() {
            if !c.is_alphanumeric() && c != '_' && c != '-' {
                return Err(ClawMeshError::with_message(
                    ErrorCode::InvalidCharacters,
                    "Username can only contain letters, numbers, underscores, and hyphens",
                ).with_field("username"));
            }
        }

        Ok(())
    }

    /// Validate nickname (optional, less strict)
    pub fn validate_nickname(nickname: &str) -> ClawMeshResult<()> {
        if nickname.is_empty() {
            return Ok(()); // Empty nickname is allowed (to clear it)
        }
        validate_length(nickname, "nickname", 1, MAX_NICKNAME_LENGTH)?;
        Ok(())
    }

    /// Validate pagination parameters
    pub fn validate_pagination(page: i32, limit: i32) -> ClawMeshResult<(i32, i32)> {
        let page = if page < 1 { 1 } else { page };
        let limit = limit.clamp(1, 100);
        Ok((page, limit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::validation::*;

    #[test]
    fn test_error_code_http_status() {
        assert_eq!(ErrorCode::Unauthenticated.http_status(), StatusCode::UNAUTHORIZED);
        assert_eq!(ErrorCode::NotFound.http_status(), StatusCode::NOT_FOUND);
        assert_eq!(ErrorCode::InvalidInput.http_status(), StatusCode::BAD_REQUEST);
        assert_eq!(ErrorCode::InternalError.http_status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_creation() {
        let err = ClawMeshError::new(ErrorCode::NotFound);
        assert_eq!(err.code, ErrorCode::NotFound);
        assert_eq!(err.message, "Resource not found");
    }

    #[test]
    fn test_error_with_details() {
        let err = ClawMeshError::with_message(ErrorCode::InvalidInput, "Custom message")
            .with_details("Additional info")
            .with_field("username");
        
        assert_eq!(err.message, "Custom message");
        assert_eq!(err.details, Some("Additional info".to_string()));
        assert_eq!(err.field, Some("username".to_string()));
    }

    #[test]
    fn test_validate_non_empty() {
        assert!(require_non_empty("hello", "field").is_ok());
        assert!(require_non_empty("", "field").is_err());
        assert!(require_non_empty("   ", "field").is_err());
    }

    #[test]
    fn test_validate_length() {
        assert!(validate_length("hello", "field", 1, 10).is_ok());
        assert!(validate_length("hi", "field", 3, 10).is_err());
        assert!(validate_length("hello world!", "field", 1, 5).is_err());
    }

    #[test]
    fn test_validate_user_id() {
        assert!(validate_user_id(1, "user_id").is_ok());
        assert!(validate_user_id(0, "user_id").is_err());
        assert!(validate_user_id(-1, "user_id").is_err());
    }

    #[test]
    fn test_validate_not_self() {
        assert!(validate_not_self(1, 2, "friend").is_ok());
        assert!(validate_not_self(1, 1, "friend").is_err());
    }

    #[test]
    fn test_validate_username() {
        assert!(validate_username("valid_user").is_ok());
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("my-agent").is_ok());
        assert!(validate_username("ab").is_err()); // too short
        assert!(validate_username("_invalid").is_err()); // starts with underscore
        assert!(validate_username("invalid user").is_err()); // contains space
    }

    #[test]
    fn test_validate_pagination() {
        assert_eq!(validate_pagination(1, 20).unwrap(), (1, 20));
        assert_eq!(validate_pagination(0, 20).unwrap(), (1, 20)); // page corrected
        assert_eq!(validate_pagination(1, 200).unwrap(), (1, 100)); // limit clamped
        assert_eq!(validate_pagination(1, 0).unwrap(), (1, 1)); // limit corrected
    }
}
