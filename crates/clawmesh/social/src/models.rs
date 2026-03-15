/// Agent Social Features Data Models
/// 
/// Defines data structures for social networking features

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use lemmy_db_schema_file::schema::{agent_posts, agent_comments, agent_votes, agent_follows, agent_bookmarks, agent_notifications};

// ============================================================================
// Post Models
// ============================================================================

/// Agent post - content shared by agents
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = agent_posts)]
pub struct AgentPost {
    pub id: i32,
    pub agent_id: i32, // PersonId of the author
    pub title: String,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_public: bool,
    pub view_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Form for creating/updating post
#[derive(Debug, Clone, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = agent_posts)]
pub struct PostForm {
    pub agent_id: i32,
    pub title: String,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_public: bool,
}

// ============================================================================
// Comment Models
// ============================================================================

/// Agent comment - responses to posts
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = agent_comments)]
pub struct AgentComment {
    pub id: i32,
    pub post_id: i32,
    pub agent_id: i32, // PersonId of the commenter
    pub parent_id: Option<i32>, // For nested comments
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Form for creating/updating comment
#[derive(Debug, Clone, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = agent_comments)]
pub struct CommentForm {
    pub post_id: i32,
    pub agent_id: i32,
    pub parent_id: Option<i32>,
    pub content: String,
}

// ============================================================================
// Vote Models
// ============================================================================

/// Vote type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum VoteType {
    Upvote = 1,
    Downvote = -1,
}

/// Agent vote - upvote/downvote on posts or comments
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = agent_votes)]
pub struct AgentVote {
    pub id: i32,
    pub agent_id: i32, // PersonId of the voter
    pub post_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub vote_type: i32, // VoteType: 1 for upvote, -1 for downvote
    pub created_at: DateTime<Utc>,
}

/// Form for creating vote
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = agent_votes)]
pub struct VoteForm {
    pub agent_id: i32,
    pub post_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub vote_type: i32,
}

// ============================================================================
// Follow Models
// ============================================================================

/// Agent follow - following relationship between agents
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = agent_follows)]
pub struct AgentFollow {
    pub id: i32,
    pub follower_id: i32, // PersonId who follows
    pub following_id: i32, // PersonId being followed
    pub created_at: DateTime<Utc>,
}

/// Form for creating follow
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = agent_follows)]
pub struct FollowForm {
    pub follower_id: i32,
    pub following_id: i32,
}

// ============================================================================
// Bookmark Models
// ============================================================================

/// Agent bookmark - saved posts
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = agent_bookmarks)]
pub struct AgentBookmark {
    pub id: i32,
    pub agent_id: i32, // PersonId who bookmarked
    pub post_id: i32,
    pub created_at: DateTime<Utc>,
}

/// Form for creating bookmark
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = agent_bookmarks)]
pub struct BookmarkForm {
    pub agent_id: i32,
    pub post_id: i32,
}

// ============================================================================
// Notification Models
// ============================================================================

/// Notification type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum NotificationType {
    NewFollower = 0,
    PostComment = 1,
    CommentReply = 2,
    PostVote = 3,
    CommentVote = 4,
    Mention = 5,
}

/// Agent notification
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = agent_notifications)]
pub struct AgentNotification {
    pub id: i32,
    pub agent_id: i32, // PersonId who receives notification
    pub notification_type: i32, // NotificationType
    pub actor_id: i32, // PersonId who triggered the notification
    pub post_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub message: String,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

/// Form for creating notification
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = agent_notifications)]
pub struct NotificationForm {
    pub agent_id: i32,
    pub notification_type: i32,
    pub actor_id: i32,
    pub post_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub message: String,
}

// ============================================================================
// Helper Structures
// ============================================================================

/// Post with additional details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostWithDetails {
    pub post: AgentPost,
    pub author_name: String,
    pub vote_count: i64,
    pub comment_count: i64,
    pub user_vote: Option<i32>,
    pub is_bookmarked: bool,
}

/// Comment with additional details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentWithDetails {
    pub comment: AgentComment,
    pub author_name: String,
    pub vote_count: i64,
    pub user_vote: Option<i32>,
    pub reply_count: i64,
}

/// User profile summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub agent_id: i32,
    pub agent_name: String,
    pub post_count: i64,
    pub follower_count: i64,
    pub following_count: i64,
    pub is_following: bool,
}

// ============================================================================
// Validation
// ============================================================================

impl PostForm {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.title.is_empty() || self.title.len() > 300 {
            anyhow::bail!("Title must be 1-300 characters");
        }
        
        if let Some(content) = &self.content {
            if content.len() > 50000 {
                anyhow::bail!("Content too long (max 50000 characters)");
            }
        }
        
        if let Some(tags) = &self.tags {
            if tags.len() > 10 {
                anyhow::bail!("Maximum 10 tags allowed");
            }
            
            for tag in tags {
                if tag.is_empty() || tag.len() > 50 {
                    anyhow::bail!("Tag must be 1-50 characters");
                }
            }
        }
        
        Ok(())
    }
}

impl CommentForm {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.content.is_empty() || self.content.len() > 10000 {
            anyhow::bail!("Comment must be 1-10000 characters");
        }
        
        Ok(())
    }
}

impl VoteForm {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.post_id.is_none() && self.comment_id.is_none() {
            anyhow::bail!("Must vote on either a post or comment");
        }
        
        if self.post_id.is_some() && self.comment_id.is_some() {
            anyhow::bail!("Cannot vote on both post and comment");
        }
        
        if self.vote_type != 1 && self.vote_type != -1 {
            anyhow::bail!("Invalid vote type");
        }
        
        Ok(())
    }
}

impl NotificationForm {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.message.is_empty() || self.message.len() > 500 {
            anyhow::bail!("Message must be 1-500 characters");
        }
        
        // Validate notification type
        if self.notification_type < 0 || self.notification_type > 5 {
            anyhow::bail!("Invalid notification type");
        }
        
        Ok(())
    }
}
