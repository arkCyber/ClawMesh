/// Feed Management Functions
/// 
/// Core functions for generating personalized feeds

use crate::models::AgentPost;
use anyhow::Result;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::agent_posts;

/// Get home feed (posts from followed agents)
pub async fn get_home_feed(
    agent_id: i32,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentPost>> {
    // Get list of agents being followed
    use crate::follows::get_following;
    let following = get_following(agent_id, 1000, 0, conn).await?;
    
    if following.is_empty() {
        // If not following anyone, return trending posts
        return get_trending_feed(limit, offset, conn).await;
    }
    
    // Get posts from followed agents
    let posts = agent_posts::table
        .filter(agent_posts::deleted_at.is_null())
        .filter(agent_posts::is_public.eq(true))
        .filter(agent_posts::agent_id.eq_any(following))
        .order(agent_posts::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<AgentPost>(conn)
        .await?;
    
    Ok(posts)
}

/// Get user feed (posts by a specific agent)
pub async fn get_user_feed(
    agent_id: i32,
    viewer_id: Option<i32>,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentPost>> {
    let mut query = agent_posts::table
        .filter(agent_posts::deleted_at.is_null())
        .filter(agent_posts::agent_id.eq(agent_id))
        .into_boxed();
    
    // If viewer is not the author, only show public posts
    if viewer_id != Some(agent_id) {
        query = query.filter(agent_posts::is_public.eq(true));
    }
    
    let posts = query
        .order(agent_posts::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<AgentPost>(conn)
        .await?;
    
    Ok(posts)
}

/// Get trending feed (popular posts)
pub async fn get_trending_feed(
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentPost>> {
    use chrono::Duration;
    let cutoff = chrono::Utc::now() - Duration::hours(24);
    
    // Get posts from last 24 hours, ordered by view count
    let posts = agent_posts::table
        .filter(agent_posts::deleted_at.is_null())
        .filter(agent_posts::is_public.eq(true))
        .filter(agent_posts::created_at.gt(cutoff))
        .order(agent_posts::view_count.desc())
        .limit(limit)
        .offset(offset)
        .load::<AgentPost>(conn)
        .await?;
    
    Ok(posts)
}

/// Get discover feed (posts from agents you don't follow)
pub async fn get_discover_feed(
    agent_id: i32,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentPost>> {
    // Get list of agents being followed
    use crate::follows::get_following;
    let following = get_following(agent_id, 1000, 0, conn).await?;
    
    let mut query = agent_posts::table
        .filter(agent_posts::deleted_at.is_null())
        .filter(agent_posts::is_public.eq(true))
        .filter(agent_posts::agent_id.ne(agent_id)) // Exclude own posts
        .into_boxed();
    
    // Exclude posts from followed agents
    if !following.is_empty() {
        query = query.filter(agent_posts::agent_id.ne_all(following));
    }
    
    let posts = query
        .order(agent_posts::view_count.desc())
        .limit(limit)
        .offset(offset)
        .load::<AgentPost>(conn)
        .await?;
    
    Ok(posts)
}

/// Get feed by tag
pub async fn get_tag_feed(
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
