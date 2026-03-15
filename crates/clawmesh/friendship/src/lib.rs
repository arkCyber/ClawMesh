//! ClawMesh Friendship System
//!
//! Provides friend relationship management, friend requests,
//! and contact communication capabilities.

pub mod friendship;
pub mod request;
pub mod block;

pub use friendship::{Friendship, FriendshipForm, FriendshipStatus};
pub use request::{FriendRequest, FriendRequestForm, RequestStatus};
pub use block::{UserBlock, UserBlockForm};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Friendship configuration
#[derive(Debug, Clone)]
pub struct FriendshipConfig {
    /// Maximum number of friends per user
    pub max_friends: usize,
    /// Maximum pending friend requests
    pub max_pending_requests: usize,
    /// Request expiration days
    pub request_expiration_days: i32,
    /// Allow friend requests from strangers
    pub allow_stranger_requests: bool,
}

impl Default for FriendshipConfig {
    fn default() -> Self {
        Self {
            max_friends: 5000,
            max_pending_requests: 100,
            request_expiration_days: 30,
            allow_stranger_requests: true,
        }
    }
}

/// Friend visibility level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FriendVisibility {
    /// Visible to everyone
    Public,
    /// Visible to friends only
    FriendsOnly,
    /// Visible to mutual friends only
    MutualFriends,
    /// Hidden from everyone
    Private,
}

impl Default for FriendVisibility {
    fn default() -> Self {
        Self::FriendsOnly
    }
}

/// Online status for friends
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OnlineStatus {
    /// User is online
    Online,
    /// User is away
    Away,
    /// User is busy (do not disturb)
    Busy,
    /// User appears offline
    Invisible,
    /// User is offline
    Offline,
}

impl Default for OnlineStatus {
    fn default() -> Self {
        Self::Offline
    }
}

/// Friend info for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendInfo {
    /// Friend's user ID
    pub user_id: i32,
    /// Friend's username
    pub username: String,
    /// Friend's display name
    pub display_name: Option<String>,
    /// Friend's avatar URL
    pub avatar: Option<String>,
    /// Online status
    pub online_status: OnlineStatus,
    /// Last seen time
    pub last_seen: Option<DateTime<Utc>>,
    /// Friendship established time
    pub friends_since: DateTime<Utc>,
    /// Custom nickname for this friend
    pub nickname: Option<String>,
    /// Friend notes
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FriendshipConfig::default();
        assert_eq!(config.max_friends, 5000);
        assert_eq!(config.max_pending_requests, 100);
        assert!(config.allow_stranger_requests);
    }

    #[test]
    fn test_online_status() {
        let status = OnlineStatus::Online;
        assert_eq!(status, OnlineStatus::Online);
    }

    #[test]
    fn test_friend_visibility() {
        let visibility = FriendVisibility::default();
        assert_eq!(visibility, FriendVisibility::FriendsOnly);
    }
}
