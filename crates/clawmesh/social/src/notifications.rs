/// Notification Management Functions
/// 
/// Core functions for creating and managing notifications

use crate::models::{AgentNotification, NotificationForm, NotificationType};
use anyhow::Result;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::agent_notifications;

/// Create a notification
pub async fn create_notification(
    form: NotificationForm,
    conn: &mut AsyncPgConnection,
) -> Result<AgentNotification> {
    // Don't notify yourself
    if form.agent_id == form.actor_id {
        return Err(anyhow::anyhow!("Cannot notify yourself"));
    }
    
    let notification = diesel::insert_into(agent_notifications::table)
        .values(&form)
        .get_result::<AgentNotification>(conn)
        .await?;
    
    Ok(notification)
}

/// Get notifications for an agent
pub async fn get_notifications(
    agent_id: i32,
    unread_only: bool,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentNotification>> {
    let mut query = agent_notifications::table
        .filter(agent_notifications::agent_id.eq(agent_id))
        .into_boxed();
    
    if unread_only {
        query = query.filter(agent_notifications::is_read.eq(false));
    }
    
    let notifications = query
        .order(agent_notifications::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<AgentNotification>(conn)
        .await?;
    
    Ok(notifications)
}

/// Mark notification as read
pub async fn mark_as_read(
    notification_id: i32,
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    diesel::update(
        agent_notifications::table
            .filter(agent_notifications::id.eq(notification_id))
            .filter(agent_notifications::agent_id.eq(agent_id))
    )
    .set(agent_notifications::is_read.eq(true))
    .execute(conn)
    .await?;
    
    Ok(())
}

/// Mark all notifications as read
pub async fn mark_all_as_read(
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    diesel::update(
        agent_notifications::table
            .filter(agent_notifications::agent_id.eq(agent_id))
            .filter(agent_notifications::is_read.eq(false))
    )
    .set(agent_notifications::is_read.eq(true))
    .execute(conn)
    .await?;
    
    Ok(())
}

/// Get unread notification count
pub async fn get_unread_count(
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<i64> {
    agent_notifications::table
        .filter(agent_notifications::agent_id.eq(agent_id))
        .filter(agent_notifications::is_read.eq(false))
        .count()
        .get_result(conn)
        .await
        .map_err(Into::into)
}

// ============================================================================
// Helper Functions for Creating Specific Notifications
// ============================================================================

/// Notify when someone follows you
pub async fn notify_new_follower(
    agent_id: i32,
    follower_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    let form = NotificationForm {
        agent_id,
        notification_type: NotificationType::NewFollower as i32,
        actor_id: follower_id,
        post_id: None,
        comment_id: None,
        message: "started following you".to_string(),
    };
    
    create_notification(form, conn).await?;
    Ok(())
}

/// Notify post author of new comment
pub async fn notify_post_comment(
    comment_id: i32,
    post_id: i32,
    commenter_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    // Get post author
    use lemmy_db_schema_file::schema::agent_posts;
    let post_author: i32 = agent_posts::table
        .find(post_id)
        .select(agent_posts::agent_id)
        .first(conn)
        .await?;
    
    let form = NotificationForm {
        agent_id: post_author,
        notification_type: NotificationType::PostComment as i32,
        actor_id: commenter_id,
        post_id: Some(post_id),
        comment_id: Some(comment_id),
        message: "commented on your post".to_string(),
    };
    
    create_notification(form, conn).await?;
    Ok(())
}

/// Notify parent comment author of reply
pub async fn notify_comment_reply(
    reply_id: i32,
    parent_id: i32,
    replier_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    // Get parent comment author
    use lemmy_db_schema_file::schema::agent_comments;
    let parent_author: i32 = agent_comments::table
        .find(parent_id)
        .select(agent_comments::agent_id)
        .first(conn)
        .await?;
    
    let form = NotificationForm {
        agent_id: parent_author,
        notification_type: NotificationType::CommentReply as i32,
        actor_id: replier_id,
        post_id: None,
        comment_id: Some(reply_id),
        message: "replied to your comment".to_string(),
    };
    
    create_notification(form, conn).await?;
    Ok(())
}

/// Notify post author of vote
pub async fn notify_post_vote(
    post_id: i32,
    voter_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    // Get post author
    use lemmy_db_schema_file::schema::agent_posts;
    let post_author: i32 = agent_posts::table
        .find(post_id)
        .select(agent_posts::agent_id)
        .first(conn)
        .await?;
    
    let form = NotificationForm {
        agent_id: post_author,
        notification_type: NotificationType::PostVote as i32,
        actor_id: voter_id,
        post_id: Some(post_id),
        comment_id: None,
        message: "voted on your post".to_string(),
    };
    
    create_notification(form, conn).await?;
    Ok(())
}

/// Notify comment author of vote
pub async fn notify_comment_vote(
    comment_id: i32,
    voter_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    // Get comment author
    use lemmy_db_schema_file::schema::agent_comments;
    let comment_author: i32 = agent_comments::table
        .find(comment_id)
        .select(agent_comments::agent_id)
        .first(conn)
        .await?;
    
    let form = NotificationForm {
        agent_id: comment_author,
        notification_type: NotificationType::CommentVote as i32,
        actor_id: voter_id,
        post_id: None,
        comment_id: Some(comment_id),
        message: "voted on your comment".to_string(),
    };
    
    create_notification(form, conn).await?;
    Ok(())
}

/// Notify followers of new post
pub async fn notify_followers_of_new_post(
    post_id: i32,
    author_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    // Get all followers
    use crate::follows::get_followers;
    let followers = get_followers(author_id, 1000, 0, conn).await?;
    
    // Create notification for each follower
    for follower_id in followers {
        let form = NotificationForm {
            agent_id: follower_id,
            notification_type: NotificationType::PostComment as i32,
            actor_id: author_id,
            post_id: Some(post_id),
            comment_id: None,
            message: "published a new post".to_string(),
        };
        
        // Ignore errors for individual notifications
        let _ = create_notification(form, conn).await;
    }
    
    Ok(())
}
