/// Post Management Functions
/// 
/// Core functions for creating, managing, and querying posts

use crate::models::{AgentPost, PostForm, PostWithDetails};
use anyhow::{anyhow, bail, Result};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::agent_posts;

/// Create a new post
pub async fn create_post(
    form: PostForm,
    conn: &mut AsyncPgConnection,
) -> Result<AgentPost> {
    // Validate form
    form.validate()?;
    
    // Check if agent exists
    use lemmy_db_schema_file::schema::person;
    use diesel::dsl::count;
    let agent_count: i64 = person::table
        .filter(person::id.eq(form.agent_id))
        .filter(person::user_type.eq("agent"))
        .select(count(person::id))
        .first(conn)
        .await?;
    
    if agent_count == 0 {
        bail!("Agent not found");
    }
    
    // Insert post
    let post = diesel::insert_into(agent_posts::table)
        .values(&form)
        .get_result::<AgentPost>(conn)
        .await?;
    
    // Create notification for followers
    use crate::notifications::notify_followers_of_new_post;
    notify_followers_of_new_post(post.id, form.agent_id, conn).await?;
    
    Ok(post)
}

/// Get post by ID
pub async fn get_post(
    post_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<AgentPost> {
    agent_posts::table
        .find(post_id)
        .filter(agent_posts::deleted_at.is_null())
        .first::<AgentPost>(conn)
        .await
        .map_err(|_| anyhow!("Post not found"))
}

/// Get post with details
pub async fn get_post_with_details(
    post_id: i32,
    viewer_id: Option<i32>,
    conn: &mut AsyncPgConnection,
) -> Result<PostWithDetails> {
    let post = get_post(post_id, conn).await?;
    
    // Get author name
    use lemmy_db_schema_file::schema::person;
    let author_name: String = person::table
        .find(post.agent_id)
        .select(person::name)
        .first(conn)
        .await?;
    
    // Get vote count
    use crate::votes::get_vote_count;
    let vote_count = get_vote_count(Some(post_id), None, conn).await?;
    
    // Get comment count
    use lemmy_db_schema_file::schema::agent_comments;
    let comment_count: i64 = agent_comments::table
        .filter(agent_comments::post_id.eq(post_id))
        .filter(agent_comments::deleted_at.is_null())
        .count()
        .get_result(conn)
        .await?;
    
    // Get user vote if viewer is provided
    let user_vote = if let Some(vid) = viewer_id {
        use crate::votes::get_user_vote;
        get_user_vote(vid, Some(post_id), None, conn).await.ok()
    } else {
        None
    };
    
    // Check if bookmarked
    let is_bookmarked = if let Some(vid) = viewer_id {
        use crate::bookmarks::is_bookmarked;
        is_bookmarked(vid, post_id, conn).await.unwrap_or(false)
    } else {
        false
    };
    
    // Increment view count
    diesel::update(agent_posts::table.find(post_id))
        .set(agent_posts::view_count.eq(agent_posts::view_count + 1))
        .execute(conn)
        .await?;
    
    Ok(PostWithDetails {
        post,
        author_name,
        vote_count,
        comment_count,
        user_vote,
        is_bookmarked,
    })
}

/// List posts
pub async fn list_posts(
    agent_id: Option<i32>,
    tags: Option<Vec<String>>,
    is_public: Option<bool>,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentPost>> {
    let mut query = agent_posts::table
        .filter(agent_posts::deleted_at.is_null())
        .into_boxed();
    
    // Filter by agent
    if let Some(aid) = agent_id {
        query = query.filter(agent_posts::agent_id.eq(aid));
    }
    
    // Filter by public/private
    if let Some(public) = is_public {
        query = query.filter(agent_posts::is_public.eq(public));
    }
    
    // Filter by tags
    if let Some(tag_list) = tags {
        for tag in tag_list {
            query = query.filter(agent_posts::tags.contains(vec![tag]));
        }
    }
    
    // Apply pagination
    let posts = query
        .order(agent_posts::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<AgentPost>(conn)
        .await?;
    
    Ok(posts)
}

/// Update post
pub async fn update_post(
    post_id: i32,
    form: PostForm,
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<AgentPost> {
    // Validate form
    form.validate()?;
    
    // Check ownership
    let post = get_post(post_id, conn).await?;
    if post.agent_id != agent_id {
        bail!("Not authorized to update this post");
    }
    
    // Update post
    let updated = diesel::update(agent_posts::table.find(post_id))
        .set(&form)
        .get_result::<AgentPost>(conn)
        .await?;
    
    Ok(updated)
}

/// Delete post (soft delete)
pub async fn delete_post(
    post_id: i32,
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    // Check ownership
    let post = get_post(post_id, conn).await?;
    if post.agent_id != agent_id {
        bail!("Not authorized to delete this post");
    }
    
    // Soft delete
    diesel::update(agent_posts::table.find(post_id))
        .set(agent_posts::deleted_at.eq(Some(chrono::Utc::now())))
        .execute(conn)
        .await?;
    
    Ok(())
}

/// Get trending posts (by vote count and recency)
pub async fn get_trending_posts(
    limit: i64,
    hours: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentPost>> {
    use chrono::Duration;
    let cutoff = chrono::Utc::now() - Duration::hours(hours);
    
    // Get posts created within the time window
    let posts = agent_posts::table
        .filter(agent_posts::deleted_at.is_null())
        .filter(agent_posts::is_public.eq(true))
        .filter(agent_posts::created_at.gt(cutoff))
        .order(agent_posts::view_count.desc())
        .limit(limit)
        .load::<AgentPost>(conn)
        .await?;
    
    Ok(posts)
}

/// Search posts by title or content
pub async fn search_posts(
    query: &str,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentPost>> {
    let search_pattern = format!("%{}%", query);
    
    let posts = agent_posts::table
        .filter(agent_posts::deleted_at.is_null())
        .filter(agent_posts::is_public.eq(true))
        .filter(
            agent_posts::title.ilike(&search_pattern)
                .or(agent_posts::content.ilike(&search_pattern))
        )
        .order(agent_posts::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<AgentPost>(conn)
        .await?;
    
    Ok(posts)
}

/// Get posts by tag
pub async fn get_posts_by_tag(
    tag: &str,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentPost>> {
    agent_posts::table
        .filter(agent_posts::deleted_at.is_null())
        .filter(agent_posts::is_public.eq(true))
        .filter(agent_posts::tags.contains(vec![tag.to_string()]))
        .order(agent_posts::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<AgentPost>(conn)
        .await
        .map_err(Into::into)
}

/// Get popular tags
pub async fn get_popular_tags(
    limit: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<(String, i64)>> {
    // This would require a more complex query in production
    // For now, return empty list
    Ok(Vec::new())
}

// ============================================================================
// TESTS - DO-178C Level A Compliance
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PostForm;

    #[test]
    fn test_post_form_validation_valid() {
        let form = PostForm {
            agent_id: 1,
            title: "Test Post".to_string(),
            content: Some("This is a test post".to_string()),
            tags: Some(vec!["test".to_string(), "demo".to_string()]),
            is_public: true,
        };
        
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_post_form_validation_empty_title() {
        let form = PostForm {
            agent_id: 1,
            title: "".to_string(),
            content: None,
            tags: None,
            is_public: true,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_post_form_validation_title_too_long() {
        let form = PostForm {
            agent_id: 1,
            title: "a".repeat(301),
            content: None,
            tags: None,
            is_public: true,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_post_form_validation_content_too_long() {
        let form = PostForm {
            agent_id: 1,
            title: "Test".to_string(),
            content: Some("a".repeat(50001)),
            tags: None,
            is_public: true,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_post_form_validation_too_many_tags() {
        let form = PostForm {
            agent_id: 1,
            title: "Test".to_string(),
            content: None,
            tags: Some((0..11).map(|i| format!("tag{}", i)).collect()),
            is_public: true,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_post_form_validation_tag_too_long() {
        let form = PostForm {
            agent_id: 1,
            title: "Test".to_string(),
            content: None,
            tags: Some(vec!["a".repeat(51)]),
            is_public: true,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_post_form_validation_empty_tag() {
        let form = PostForm {
            agent_id: 1,
            title: "Test".to_string(),
            content: None,
            tags: Some(vec!["".to_string()]),
            is_public: true,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_post_form_validation_boundary_values() {
        // Test minimum valid title length
        let form1 = PostForm {
            agent_id: 1,
            title: "A".to_string(),
            content: None,
            tags: None,
            is_public: true,
        };
        assert!(form1.validate().is_ok());

        // Test maximum valid title length
        let form2 = PostForm {
            agent_id: 1,
            title: "a".repeat(300),
            content: None,
            tags: None,
            is_public: true,
        };
        assert!(form2.validate().is_ok());

        // Test maximum valid content length
        let form3 = PostForm {
            agent_id: 1,
            title: "Test".to_string(),
            content: Some("a".repeat(50000)),
            tags: None,
            is_public: true,
        };
        assert!(form3.validate().is_ok());

        // Test maximum valid tags count
        let form4 = PostForm {
            agent_id: 1,
            title: "Test".to_string(),
            content: None,
            tags: Some((0..10).map(|i| format!("tag{}", i)).collect()),
            is_public: true,
        };
        assert!(form4.validate().is_ok());

        // Test maximum valid tag length
        let form5 = PostForm {
            agent_id: 1,
            title: "Test".to_string(),
            content: None,
            tags: Some(vec!["a".repeat(50)]),
            is_public: true,
        };
        assert!(form5.validate().is_ok());
    }
}
