//! Friend request management
//!
//! Handles friend request creation, acceptance, and rejection

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Friend request status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RequestStatus {
    /// Request is pending
    Pending,
    /// Request was accepted
    Accepted,
    /// Request was rejected
    Rejected,
    /// Request was cancelled by sender
    Cancelled,
    /// Request expired
    Expired,
}

impl Default for RequestStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Friend request between two users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRequest {
    /// Unique request ID
    pub id: i64,
    /// User who sent the request
    pub sender_id: i32,
    /// User who received the request
    pub recipient_id: i32,
    /// Request status
    pub status: RequestStatus,
    /// Optional message with the request
    pub message: Option<String>,
    /// When the request was created
    pub created_at: DateTime<Utc>,
    /// When the request was responded to
    pub responded_at: Option<DateTime<Utc>>,
    /// When the request expires
    pub expires_at: Option<DateTime<Utc>>,
}

impl FriendRequest {
    /// Create a new friend request
    pub fn new(sender_id: i32, recipient_id: i32) -> Self {
        Self {
            id: 0,
            sender_id,
            recipient_id,
            status: RequestStatus::Pending,
            message: None,
            created_at: Utc::now(),
            responded_at: None,
            expires_at: None,
        }
    }

    /// Create a new friend request with a message
    pub fn with_message(sender_id: i32, recipient_id: i32, message: String) -> Self {
        let mut request = Self::new(sender_id, recipient_id);
        request.message = Some(message);
        request
    }

    /// Check if the request is pending
    pub fn is_pending(&self) -> bool {
        self.status == RequestStatus::Pending
    }

    /// Check if the request has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Accept the request
    pub fn accept(&mut self) {
        self.status = RequestStatus::Accepted;
        self.responded_at = Some(Utc::now());
    }

    /// Reject the request
    pub fn reject(&mut self) {
        self.status = RequestStatus::Rejected;
        self.responded_at = Some(Utc::now());
    }

    /// Cancel the request (by sender)
    pub fn cancel(&mut self) {
        self.status = RequestStatus::Cancelled;
        self.responded_at = Some(Utc::now());
    }

    /// Mark as expired
    pub fn mark_expired(&mut self) {
        self.status = RequestStatus::Expired;
    }
}

/// Form for creating a friend request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRequestForm {
    /// Sender user ID
    pub sender_id: i32,
    /// Recipient user ID
    pub recipient_id: i32,
    /// Optional message
    pub message: Option<String>,
    /// Expiration days (optional)
    pub expiration_days: Option<i32>,
}

impl FriendRequestForm {
    /// Create a new friend request form
    pub fn new(sender_id: i32, recipient_id: i32) -> Self {
        Self {
            sender_id,
            recipient_id,
            message: None,
            expiration_days: None,
        }
    }

    /// Add a message to the request
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    /// Set expiration days
    pub fn with_expiration(mut self, days: i32) -> Self {
        self.expiration_days = Some(days);
        self
    }
}

/// Form for responding to a friend request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRequestResponse {
    /// Request ID
    pub request_id: i64,
    /// Whether to accept the request
    pub accept: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_friend_request_creation() {
        let request = FriendRequest::new(1, 2);
        assert_eq!(request.sender_id, 1);
        assert_eq!(request.recipient_id, 2);
        assert!(request.is_pending());
    }

    #[test]
    fn test_friend_request_with_message() {
        let request = FriendRequest::with_message(1, 2, "Hello!".to_string());
        assert_eq!(request.message, Some("Hello!".to_string()));
    }

    #[test]
    fn test_accept_request() {
        let mut request = FriendRequest::new(1, 2);
        request.accept();
        assert_eq!(request.status, RequestStatus::Accepted);
        assert!(request.responded_at.is_some());
    }

    #[test]
    fn test_reject_request() {
        let mut request = FriendRequest::new(1, 2);
        request.reject();
        assert_eq!(request.status, RequestStatus::Rejected);
    }

    #[test]
    fn test_cancel_request() {
        let mut request = FriendRequest::new(1, 2);
        request.cancel();
        assert_eq!(request.status, RequestStatus::Cancelled);
    }

    #[test]
    fn test_request_form_builder() {
        let form = FriendRequestForm::new(1, 2)
            .with_message("Let's be friends!".to_string())
            .with_expiration(30);
        
        assert_eq!(form.sender_id, 1);
        assert_eq!(form.recipient_id, 2);
        assert_eq!(form.message, Some("Let's be friends!".to_string()));
        assert_eq!(form.expiration_days, Some(30));
    }
}
