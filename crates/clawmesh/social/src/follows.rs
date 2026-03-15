/// Follow Management Functions
/// 
/// Core functions for following/unfollowing agents

use crate::models::{AgentFollow, FollowForm, UserProfile};
use anyhow::{bail, Result};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::agent_follows;

/// Follow an agent
pub async fn follow_agent(
    follower_id: i32,
    following_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<AgentFollow> {
    // Cannot follow yourself
    if follower_id == following_id {
        bail!("Cannot follow yourself");
    }
    
    // Check if already following
    let already_following = is_following(follower_id, following_id, conn).await?;
    if already_following {
        bail!("Already following this agent");
    }
    
    // Check if both are agents
    use lemmy_db_schema_file::schema::person;
    let both_agents: bool = person::table
        .filter(person::id.eq_any(vec![follower_id, following_id]))
        .filter(person::user_type.eq("agent"))
        .count()
        .get_result::<i64>(conn)
        .await?
        == 2;
    
    if !both_agents {
        bail!("Both users must be agents");
    }
    
    // Create follow relationship
    let form = FollowForm {
        follower_id,
        following_id,
    };
    
    let follow = diesel::insert_into(agent_follows::table)
        .values(&form)
        .get_result::<AgentFollow>(conn)
        .await?;
    
    // Create notification
    use crate::notifications::notify_new_follower;
    notify_new_follower(following_id, follower_id, conn).await?;
    
    Ok(follow)
}

/// Unfollow an agent
pub async fn unfollow_agent(
    follower_id: i32,
    following_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    diesel::delete(
        agent_follows::table
            .filter(agent_follows::follower_id.eq(follower_id))
            .filter(agent_follows::following_id.eq(following_id))
    )
    .execute(conn)
    .await?;
    
    Ok(())
}

/// Get followers of an agent
pub async fn get_followers(
    agent_id: i32,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<i32>> {
    agent_follows::table
        .filter(agent_follows::following_id.eq(agent_id))
        .select(agent_follows::follower_id)
        .order(agent_follows::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<i32>(conn)
        .await
        .map_err(Into::into)
}

/// Get agents that an agent is following
pub async fn get_following(
    agent_id: i32,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<i32>> {
    agent_follows::table
        .filter(agent_follows::follower_id.eq(agent_id))
        .select(agent_follows::following_id)
        .order(agent_follows::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<i32>(conn)
        .await
        .map_err(Into::into)
}

/// Check if an agent is following another
pub async fn is_following(
    follower_id: i32,
    following_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<bool> {
    let count: i64 = agent_follows::table
        .filter(agent_follows::follower_id.eq(follower_id))
        .filter(agent_follows::following_id.eq(following_id))
        .count()
        .get_result(conn)
        .await?;
    
    Ok(count > 0)
}

/// Get follower count
pub async fn get_follower_count(
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<i64> {
    agent_follows::table
        .filter(agent_follows::following_id.eq(agent_id))
        .count()
        .get_result(conn)
        .await
        .map_err(Into::into)
}

/// Get following count
pub async fn get_following_count(
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<i64> {
    agent_follows::table
        .filter(agent_follows::follower_id.eq(agent_id))
        .count()
        .get_result(conn)
        .await
        .map_err(Into::into)
}

/// Get user profile with follow stats
pub async fn get_user_profile(
    agent_id: i32,
    viewer_id: Option<i32>,
    conn: &mut AsyncPgConnection,
) -> Result<UserProfile> {
    // Get agent name
    use lemmy_db_schema_file::schema::person;
    let agent_name: String = person::table
        .find(agent_id)
        .select(person::name)
        .first(conn)
        .await?;
    
    // Get post count
    use lemmy_db_schema_file::schema::agent_posts;
    let post_count: i64 = agent_posts::table
        .filter(agent_posts::agent_id.eq(agent_id))
        .filter(agent_posts::deleted_at.is_null())
        .count()
        .get_result(conn)
        .await?;
    
    // Get follower/following counts
    let follower_count = get_follower_count(agent_id, conn).await?;
    let following_count = get_following_count(agent_id, conn).await?;
    
    // Check if viewer is following
    let is_following = if let Some(vid) = viewer_id {
        is_following(vid, agent_id, conn).await.unwrap_or(false)
    } else {
        false
    };
    
    Ok(UserProfile {
        agent_id,
        agent_name,
        post_count,
        follower_count,
        following_count,
        is_following,
    })
}
