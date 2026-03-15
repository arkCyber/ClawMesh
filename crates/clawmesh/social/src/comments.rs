/// Comment Management Functions
/// 
/// Core functions for creating, managing, and querying comments

use crate::models::{AgentComment, CommentForm, CommentWithDetails};
use anyhow::{anyhow, bail, Result};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::agent_comments;

/// Create a new comment
pub async fn create_comment(
    form: CommentForm,
    conn: &mut AsyncPgConnection,
) -> Result<AgentComment> {
    // Validate form
    form.validate()?;
    
    // Check if post exists
    use lemmy_db_schema_file::schema::agent_posts;
    let post_count: i64 = agent_posts::table
        .filter(agent_posts::id.eq(form.post_id))
        .filter(agent_posts::deleted_at.is_null())
        .count()
        .get_result(conn)
        .await?;
    
    if post_count == 0 {
        bail!("Post not found");
    }
    
    // Check if parent comment exists (if specified)
    if let Some(parent_id) = form.parent_id {
        let parent_count: i64 = agent_comments::table
            .filter(agent_comments::id.eq(parent_id))
            .filter(agent_comments::post_id.eq(form.post_id))
            .filter(agent_comments::deleted_at.is_null())
            .count()
            .get_result(conn)
            .await?;
        
        if parent_count == 0 {
            bail!("Parent comment not found");
        }
    }
    
    // Insert comment
    let comment = diesel::insert_into(agent_comments::table)
        .values(&form)
        .get_result::<AgentComment>(conn)
        .await?;
    
    // Create notification for post author
    use crate::notifications::notify_post_comment;
    notify_post_comment(comment.id, form.post_id, form.agent_id, conn).await?;
    
    // Create notification for parent comment author (if reply)
    if let Some(parent_id) = form.parent_id {
        use crate::notifications::notify_comment_reply;
        notify_comment_reply(comment.id, parent_id, form.agent_id, conn).await?;
    }
    
    Ok(comment)
}

/// Get comment by ID
pub async fn get_comment(
    comment_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<AgentComment> {
    agent_comments::table
        .find(comment_id)
        .filter(agent_comments::deleted_at.is_null())
        .first::<AgentComment>(conn)
        .await
        .map_err(|_| anyhow!("Comment not found"))
}

/// Get comment with details
pub async fn get_comment_with_details(
    comment_id: i32,
    viewer_id: Option<i32>,
    conn: &mut AsyncPgConnection,
) -> Result<CommentWithDetails> {
    let comment = get_comment(comment_id, conn).await?;
    
    // Get author name
    use lemmy_db_schema_file::schema::person;
    let author_name: String = person::table
        .find(comment.agent_id)
        .select(person::name)
        .first(conn)
        .await?;
    
    // Get vote count
    use crate::votes::get_vote_count;
    let vote_count = get_vote_count(None, Some(comment_id), conn).await?;
    
    // Get user vote if viewer is provided
    let user_vote = if let Some(vid) = viewer_id {
        use crate::votes::get_user_vote;
        get_user_vote(vid, None, Some(comment_id), conn).await.ok()
    } else {
        None
    };
    
    // Count replies
    let reply_count: i64 = agent_comments::table
        .filter(agent_comments::parent_id.eq(comment_id))
        .filter(agent_comments::deleted_at.is_null())
        .count()
        .get_result(conn)
        .await?;
    
    Ok(CommentWithDetails {
        comment,
        author_name,
        vote_count,
        user_vote,
        reply_count,
    })
}

/// List comments for a post
pub async fn list_comments(
    post_id: i32,
    parent_id: Option<i32>,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentComment>> {
    let mut query = agent_comments::table
        .filter(agent_comments::post_id.eq(post_id))
        .filter(agent_comments::deleted_at.is_null())
        .into_boxed();
    
    // Filter by parent (top-level or replies)
    match parent_id {
        Some(pid) => query = query.filter(agent_comments::parent_id.eq(pid)),
        None => query = query.filter(agent_comments::parent_id.is_null()),
    }
    
    // Apply pagination
    let comments = query
        .order(agent_comments::created_at.asc())
        .limit(limit)
        .offset(offset)
        .load::<AgentComment>(conn)
        .await?;
    
    Ok(comments)
}

/// Get comment tree (nested structure)
pub async fn get_comment_tree(
    post_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentComment>> {
    // Get all comments for the post
    agent_comments::table
        .filter(agent_comments::post_id.eq(post_id))
        .filter(agent_comments::deleted_at.is_null())
        .order(agent_comments::created_at.asc())
        .load::<AgentComment>(conn)
        .await
        .map_err(Into::into)
}

/// Update comment
pub async fn update_comment(
    comment_id: i32,
    content: String,
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<AgentComment> {
    // Validate content
    if content.is_empty() || content.len() > 10000 {
        bail!("Comment must be 1-10000 characters");
    }
    
    // Check ownership
    let comment = get_comment(comment_id, conn).await?;
    if comment.agent_id != agent_id {
        bail!("Not authorized to update this comment");
    }
    
    // Update comment
    let updated = diesel::update(agent_comments::table.find(comment_id))
        .set(agent_comments::content.eq(content))
        .get_result::<AgentComment>(conn)
        .await?;
    
    Ok(updated)
}

/// Delete comment (soft delete)
pub async fn delete_comment(
    comment_id: i32,
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    // Check ownership
    let comment = get_comment(comment_id, conn).await?;
    if comment.agent_id != agent_id {
        bail!("Not authorized to delete this comment");
    }
    
    // Soft delete
    diesel::update(agent_comments::table.find(comment_id))
        .set(agent_comments::deleted_at.eq(Some(chrono::Utc::now())))
        .execute(conn)
        .await?;
    
    Ok(())
}

/// Get comment count for a post
pub async fn get_comment_count(
    post_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<i64> {
    agent_comments::table
        .filter(agent_comments::post_id.eq(post_id))
        .filter(agent_comments::deleted_at.is_null())
        .count()
        .get_result(conn)
        .await
        .map_err(Into::into)
}

// ============================================================================
// TESTS - DO-178C Level A Compliance
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CommentForm;

    #[test]
    fn test_comment_form_validation_valid() {
        let form = CommentForm {
            post_id: 1,
            parent_id: None,
            agent_id: 1,
            content: "This is a test comment".to_string(),
        };
        
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_comment_form_validation_empty_content() {
        let form = CommentForm {
            post_id: 1,
            parent_id: None,
            agent_id: 1,
            content: "".to_string(),
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_comment_form_validation_content_too_long() {
        let form = CommentForm {
            post_id: 1,
            parent_id: None,
            agent_id: 1,
            content: "a".repeat(10001),
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_comment_form_validation_boundary_values() {
        // Test minimum valid content length
        let form1 = CommentForm {
            post_id: 1,
            parent_id: None,
            agent_id: 1,
            content: "A".to_string(),
        };
        assert!(form1.validate().is_ok());

        // Test maximum valid content length
        let form2 = CommentForm {
            post_id: 1,
            parent_id: Some(2),
            agent_id: 1,
            content: "a".repeat(10000),
        };
        assert!(form2.validate().is_ok());
    }
}
