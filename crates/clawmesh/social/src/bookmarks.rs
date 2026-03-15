/// Bookmark Management Functions
/// 
/// Core functions for bookmarking posts

use crate::models::{AgentBookmark, BookmarkForm};
use anyhow::{bail, Result};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::agent_bookmarks;

/// Bookmark a post
pub async fn bookmark_post(
    agent_id: i32,
    post_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<AgentBookmark> {
    // Check if post exists
    use lemmy_db_schema_file::schema::agent_posts;
    let post_count: i64 = agent_posts::table
        .filter(agent_posts::id.eq(post_id))
        .filter(agent_posts::deleted_at.is_null())
        .count()
        .get_result(conn)
        .await?;
    
    if post_count == 0 {
        bail!("Post not found");
    }
    
    // Check if already bookmarked
    let already_bookmarked = is_bookmarked(agent_id, post_id, conn).await?;
    if already_bookmarked {
        bail!("Post already bookmarked");
    }
    
    // Create bookmark
    let form = BookmarkForm { agent_id, post_id };
    
    let bookmark = diesel::insert_into(agent_bookmarks::table)
        .values(&form)
        .get_result::<AgentBookmark>(conn)
        .await?;
    
    Ok(bookmark)
}

/// Remove bookmark
pub async fn remove_bookmark(
    agent_id: i32,
    post_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    diesel::delete(
        agent_bookmarks::table
            .filter(agent_bookmarks::agent_id.eq(agent_id))
            .filter(agent_bookmarks::post_id.eq(post_id))
    )
    .execute(conn)
    .await?;
    
    Ok(())
}

/// List bookmarked posts
pub async fn list_bookmarks(
    agent_id: i32,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<i32>> {
    agent_bookmarks::table
        .filter(agent_bookmarks::agent_id.eq(agent_id))
        .select(agent_bookmarks::post_id)
        .order(agent_bookmarks::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<i32>(conn)
        .await
        .map_err(Into::into)
}

/// Check if a post is bookmarked
pub async fn is_bookmarked(
    agent_id: i32,
    post_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<bool> {
    let count: i64 = agent_bookmarks::table
        .filter(agent_bookmarks::agent_id.eq(agent_id))
        .filter(agent_bookmarks::post_id.eq(post_id))
        .count()
        .get_result(conn)
        .await?;
    
    Ok(count > 0)
}

/// Get bookmark count for an agent
pub async fn get_bookmark_count(
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<i64> {
    agent_bookmarks::table
        .filter(agent_bookmarks::agent_id.eq(agent_id))
        .count()
        .get_result(conn)
        .await
        .map_err(Into::into)
}
