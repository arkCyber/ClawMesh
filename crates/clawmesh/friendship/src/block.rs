//! User blocking functionality
//!
//! Allows users to block other users from contacting them

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User block record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBlock {
    /// Unique block ID
    pub id: i64,
    /// User who created the block
    pub blocker_id: i32,
    /// User who is blocked
    pub blocked_id: i32,
    /// Reason for blocking (optional)
    pub reason: Option<String>,
    /// When the block was created
    pub created_at: DateTime<Utc>,
}

impl UserBlock {
    /// Create a new user block
    pub fn new(blocker_id: i32, blocked_id: i32) -> Self {
        Self {
            id: 0,
            blocker_id,
            blocked_id,
            reason: None,
            created_at: Utc::now(),
        }
    }

    /// Create a new user block with reason
    pub fn with_reason(blocker_id: i32, blocked_id: i32, reason: String) -> Self {
        let mut block = Self::new(blocker_id, blocked_id);
        block.reason = Some(reason);
        block
    }
}

/// Form for creating a user block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBlockForm {
    /// User who is blocking
    pub blocker_id: i32,
    /// User to block
    pub blocked_id: i32,
    /// Optional reason
    pub reason: Option<String>,
}

impl UserBlockForm {
    /// Create a new block form
    pub fn new(blocker_id: i32, blocked_id: i32) -> Self {
        Self {
            blocker_id,
            blocked_id,
            reason: None,
        }
    }

    /// Add a reason
    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_block_creation() {
        let block = UserBlock::new(1, 2);
        assert_eq!(block.blocker_id, 1);
        assert_eq!(block.blocked_id, 2);
        assert!(block.reason.is_none());
    }

    #[test]
    fn test_user_block_with_reason() {
        let block = UserBlock::with_reason(1, 2, "Spam".to_string());
        assert_eq!(block.reason, Some("Spam".to_string()));
    }

    #[test]
    fn test_block_form() {
        let form = UserBlockForm::new(1, 2).with_reason("Harassment".to_string());
        assert_eq!(form.blocker_id, 1);
        assert_eq!(form.blocked_id, 2);
        assert_eq!(form.reason, Some("Harassment".to_string()));
    }
}
