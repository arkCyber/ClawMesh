//! Friendship Data Models
//! 
//! Aerospace-grade data models for ClawMesh friendship system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;

#[cfg(feature = "full")]
use crate::schema::{friendship, friend_request, user_block, friend_nickname};

// ============================================================================
// Friendship Models
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = friendship)]
pub struct Friendship {
    pub id: i32,
    pub user_id_1: i32,
    pub user_id_2: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = friendship)]
pub struct FriendshipForm {
    pub user_id_1: i32,
    pub user_id_2: i32,
}

impl FriendshipForm {
    /// Create a new friendship form with normalized user IDs
    /// Ensures user_id_1 < user_id_2
    pub fn new(user_a: i32, user_b: i32) -> Self {
        if user_a < user_b {
            Self {
                user_id_1: user_a,
                user_id_2: user_b,
            }
        } else {
            Self {
                user_id_1: user_b,
                user_id_2: user_a,
            }
        }
    }
}

// ============================================================================
// Friend Request Models
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FriendRequestStatus {
    Pending,
    Accepted,
    Rejected,
    Cancelled,
}

impl std::fmt::Display for FriendRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Accepted => write!(f, "accepted"),
            Self::Rejected => write!(f, "rejected"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl From<String> for FriendRequestStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "pending" => Self::Pending,
            "accepted" => Self::Accepted,
            "rejected" => Self::Rejected,
            "cancelled" => Self::Cancelled,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = friend_request)]
pub struct FriendRequest {
    pub id: i32,
    pub sender_id: i32,
    pub recipient_id: i32,
    pub message: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
}

impl FriendRequest {
    pub fn status_enum(&self) -> FriendRequestStatus {
        FriendRequestStatus::from(self.status.clone())
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = friend_request)]
pub struct FriendRequestInsertForm {
    pub sender_id: i32,
    pub recipient_id: i32,
    pub message: Option<String>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = friend_request)]
pub struct FriendRequestUpdateForm {
    pub status: Option<String>,
    pub responded_at: Option<DateTime<Utc>>,
}

// ============================================================================
// User Block Models
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = user_block)]
pub struct UserBlock {
    pub id: i32,
    pub blocker_id: i32,
    pub blocked_id: i32,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = user_block)]
pub struct UserBlockForm {
    pub blocker_id: i32,
    pub blocked_id: i32,
    pub reason: Option<String>,
}

// ============================================================================
// Friend Nickname Models
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = friend_nickname)]
pub struct FriendNickname {
    pub id: i32,
    pub user_id: i32,
    pub friend_id: i32,
    pub nickname: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = friend_nickname)]
pub struct FriendNicknameForm {
    pub user_id: i32,
    pub friend_id: i32,
    pub nickname: String,
}

// ============================================================================
// CRUD Implementations
// ============================================================================

#[cfg(feature = "full")]
impl Friendship {
    /// Create a new friendship
    pub async fn create(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        form: &FriendshipForm,
    ) -> Result<Self, diesel::result::Error> {
        use crate::schema::friendship::dsl::*;
        
        diesel::insert_into(friendship)
            .values(form)
            .get_result::<Self>(pool)
            .await
    }
    
    /// Check if two users are friends
    pub async fn are_friends(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        user_a: i32,
        user_b: i32,
    ) -> Result<bool, diesel::result::Error> {
        use crate::schema::friendship::dsl::*;
        
        let (min_id, max_id) = if user_a < user_b {
            (user_a, user_b)
        } else {
            (user_b, user_a)
        };
        
        let count: i64 = friendship
            .filter(user_id_1.eq(min_id))
            .filter(user_id_2.eq(max_id))
            .count()
            .get_result(pool)
            .await?;
        
        Ok(count > 0)
    }
    
    /// Get all friends for a user
    pub async fn get_friends(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        user_id: i32,
    ) -> Result<Vec<i32>, diesel::result::Error> {
        use crate::schema::friendship::dsl::*;
        
        let friendships: Vec<Self> = friendship
            .filter(user_id_1.eq(user_id).or(user_id_2.eq(user_id)))
            .load::<Self>(pool)
            .await?;
        
        let friend_ids: Vec<i32> = friendships
            .into_iter()
            .map(|f| {
                if f.user_id_1 == user_id {
                    f.user_id_2
                } else {
                    f.user_id_1
                }
            })
            .collect();
        
        Ok(friend_ids)
    }
    
    /// Delete a friendship
    pub async fn delete(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        user_a: i32,
        user_b: i32,
    ) -> Result<usize, diesel::result::Error> {
        use crate::schema::friendship::dsl::*;
        
        let (min_id, max_id) = if user_a < user_b {
            (user_a, user_b)
        } else {
            (user_b, user_a)
        };
        
        diesel::delete(
            friendship
                .filter(user_id_1.eq(min_id))
                .filter(user_id_2.eq(max_id))
        )
        .execute(pool)
        .await
    }
}

#[cfg(feature = "full")]
impl FriendRequest {
    /// Create a new friend request
    pub async fn create(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        form: &FriendRequestInsertForm,
    ) -> Result<Self, diesel::result::Error> {
        use crate::schema::friend_request::dsl::*;
        
        diesel::insert_into(friend_request)
            .values(form)
            .get_result::<Self>(pool)
            .await
    }
    
    /// Read a friend request by ID
    pub async fn read(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        request_id: i32,
    ) -> Result<Self, diesel::result::Error> {
        use crate::schema::friend_request::dsl::*;
        
        friend_request.find(request_id).first::<Self>(pool).await
    }
    
    /// Update a friend request
    pub async fn update(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        request_id: i32,
        form: &FriendRequestUpdateForm,
    ) -> Result<Self, diesel::result::Error> {
        use crate::schema::friend_request::dsl::*;
        
        diesel::update(friend_request.find(request_id))
            .set(form)
            .get_result::<Self>(pool)
            .await
    }
    
    /// Get pending requests for a user (incoming)
    pub async fn get_incoming_pending(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        user_id: i32,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use crate::schema::friend_request::dsl::*;
        
        friend_request
            .filter(recipient_id.eq(user_id))
            .filter(status.eq("pending"))
            .order(created_at.desc())
            .load::<Self>(pool)
            .await
    }
    
    /// Get pending requests sent by a user (outgoing)
    pub async fn get_outgoing_pending(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        user_id: i32,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use crate::schema::friend_request::dsl::*;
        
        friend_request
            .filter(sender_id.eq(user_id))
            .filter(status.eq("pending"))
            .order(created_at.desc())
            .load::<Self>(pool)
            .await
    }
    
    /// Delete a friend request
    pub async fn delete(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        request_id: i32,
    ) -> Result<usize, diesel::result::Error> {
        use crate::schema::friend_request::dsl::*;
        
        diesel::delete(friend_request.find(request_id))
            .execute(pool)
            .await
    }
}

#[cfg(feature = "full")]
impl UserBlock {
    /// Create a new block
    pub async fn create(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        form: &UserBlockForm,
    ) -> Result<Self, diesel::result::Error> {
        use crate::schema::user_block::dsl::*;
        
        diesel::insert_into(user_block)
            .values(form)
            .get_result::<Self>(pool)
            .await
    }
    
    /// Check if user is blocked
    pub async fn is_blocked(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        blocker: i32,
        blocked: i32,
    ) -> Result<bool, diesel::result::Error> {
        use crate::schema::user_block::dsl::*;
        
        let count: i64 = user_block
            .filter(blocker_id.eq(blocker))
            .filter(blocked_id.eq(blocked))
            .count()
            .get_result(pool)
            .await?;
        
        Ok(count > 0)
    }
    
    /// Get all blocked users for a blocker
    pub async fn get_blocked_users(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        blocker: i32,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use crate::schema::user_block::dsl::*;
        
        user_block
            .filter(blocker_id.eq(blocker))
            .order(created_at.desc())
            .load::<Self>(pool)
            .await
    }
    
    /// Delete a block
    pub async fn delete(
        pool: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
        blocker: i32,
        blocked: i32,
    ) -> Result<usize, diesel::result::Error> {
        use crate::schema::user_block::dsl::*;
        
        diesel::delete(
            user_block
                .filter(blocker_id.eq(blocker))
                .filter(blocked_id.eq(blocked))
        )
        .execute(pool)
        .await
    }
}
