//! Friendship relationship management
//!
//! Core friendship data structures and operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Friendship status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FriendshipStatus {
    /// Active friendship
    Active,
    /// Friendship is muted (no notifications)
    Muted,
    /// Friendship is paused
    Paused,
}

impl Default for FriendshipStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// Friendship relationship between two users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Friendship {
    /// Unique friendship ID
    pub id: i64,
    /// First user ID (lower ID)
    pub user_id_1: i32,
    /// Second user ID (higher ID)
    pub user_id_2: i32,
    /// Friendship status
    pub status: FriendshipStatus,
    /// When the friendship was established
    pub created_at: DateTime<Utc>,
    /// Last interaction time
    pub last_interaction: Option<DateTime<Utc>>,
    /// Custom nickname user_1 gave to user_2
    pub nickname_1_to_2: Option<String>,
    /// Custom nickname user_2 gave to user_1
    pub nickname_2_to_1: Option<String>,
    /// Notes from user_1 about user_2
    pub notes_1: Option<String>,
    /// Notes from user_2 about user_1
    pub notes_2: Option<String>,
}

impl Friendship {
    /// Create a new friendship between two users
    pub fn new(user_id_a: i32, user_id_b: i32) -> Self {
        // Always store with lower ID first for consistency
        let (user_id_1, user_id_2) = if user_id_a < user_id_b {
            (user_id_a, user_id_b)
        } else {
            (user_id_b, user_id_a)
        };

        Self {
            id: 0, // Will be set by database
            user_id_1,
            user_id_2,
            status: FriendshipStatus::Active,
            created_at: Utc::now(),
            last_interaction: None,
            nickname_1_to_2: None,
            nickname_2_to_1: None,
            notes_1: None,
            notes_2: None,
        }
    }

    /// Check if a user is part of this friendship
    pub fn involves_user(&self, user_id: i32) -> bool {
        self.user_id_1 == user_id || self.user_id_2 == user_id
    }

    /// Get the other user in the friendship
    pub fn get_friend_id(&self, user_id: i32) -> Option<i32> {
        if self.user_id_1 == user_id {
            Some(self.user_id_2)
        } else if self.user_id_2 == user_id {
            Some(self.user_id_1)
        } else {
            None
        }
    }

    /// Get the nickname for a friend
    pub fn get_nickname(&self, viewer_id: i32) -> Option<&str> {
        if self.user_id_1 == viewer_id {
            self.nickname_1_to_2.as_deref()
        } else if self.user_id_2 == viewer_id {
            self.nickname_2_to_1.as_deref()
        } else {
            None
        }
    }

    /// Get notes about a friend
    pub fn get_notes(&self, viewer_id: i32) -> Option<&str> {
        if self.user_id_1 == viewer_id {
            self.notes_1.as_deref()
        } else if self.user_id_2 == viewer_id {
            self.notes_2.as_deref()
        } else {
            None
        }
    }

    /// Update last interaction time
    pub fn touch(&mut self) {
        self.last_interaction = Some(Utc::now());
    }

    /// Check if friendship is active
    pub fn is_active(&self) -> bool {
        self.status == FriendshipStatus::Active
    }
}

/// Form for creating a new friendship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendshipForm {
    /// First user ID
    pub user_id_1: i32,
    /// Second user ID
    pub user_id_2: i32,
    /// Initial status
    pub status: Option<FriendshipStatus>,
}

impl FriendshipForm {
    /// Create a new friendship form
    pub fn new(user_id_a: i32, user_id_b: i32) -> Self {
        let (user_id_1, user_id_2) = if user_id_a < user_id_b {
            (user_id_a, user_id_b)
        } else {
            (user_id_b, user_id_a)
        };

        Self {
            user_id_1,
            user_id_2,
            status: None,
        }
    }
}

/// Form for updating a friendship
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FriendshipUpdateForm {
    /// New status
    pub status: Option<FriendshipStatus>,
    /// Update nickname (for user_1 to user_2)
    pub nickname_1_to_2: Option<Option<String>>,
    /// Update nickname (for user_2 to user_1)
    pub nickname_2_to_1: Option<Option<String>>,
    /// Update notes (for user_1)
    pub notes_1: Option<Option<String>>,
    /// Update notes (for user_2)
    pub notes_2: Option<Option<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_friendship_creation() {
        let friendship = Friendship::new(10, 5);
        // Should normalize order
        assert_eq!(friendship.user_id_1, 5);
        assert_eq!(friendship.user_id_2, 10);
        assert!(friendship.is_active());
    }

    #[test]
    fn test_involves_user() {
        let friendship = Friendship::new(1, 2);
        assert!(friendship.involves_user(1));
        assert!(friendship.involves_user(2));
        assert!(!friendship.involves_user(3));
    }

    #[test]
    fn test_get_friend_id() {
        let friendship = Friendship::new(1, 2);
        assert_eq!(friendship.get_friend_id(1), Some(2));
        assert_eq!(friendship.get_friend_id(2), Some(1));
        assert_eq!(friendship.get_friend_id(3), None);
    }

    #[test]
    fn test_friendship_form() {
        let form = FriendshipForm::new(10, 5);
        assert_eq!(form.user_id_1, 5);
        assert_eq!(form.user_id_2, 10);
    }
}
