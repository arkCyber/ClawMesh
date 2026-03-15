/// Vote Management Functions
/// 
/// Core functions for voting on posts and comments

use crate::models::{AgentVote, VoteForm, VoteType};
use anyhow::{bail, Result};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::agent_votes;

/// Cast a vote
pub async fn cast_vote(
    form: VoteForm,
    conn: &mut AsyncPgConnection,
) -> Result<AgentVote> {
    // Validate form
    form.validate()?;
    
    // Check if already voted
    let existing_vote = get_user_vote(
        form.agent_id,
        form.post_id,
        form.comment_id,
        conn,
    )
    .await;
    
    if let Ok(existing) = existing_vote {
        // Update existing vote
        let updated = diesel::update(agent_votes::table.find(existing))
            .set(agent_votes::vote_type.eq(form.vote_type))
            .get_result::<AgentVote>(conn)
            .await?;
        
        return Ok(updated);
    }
    
    // Insert new vote
    let vote = diesel::insert_into(agent_votes::table)
        .values(&form)
        .get_result::<AgentVote>(conn)
        .await?;
    
    // Create notification
    if form.post_id.is_some() {
        use crate::notifications::notify_post_vote;
        notify_post_vote(form.post_id.unwrap(), form.agent_id, conn).await?;
    } else if form.comment_id.is_some() {
        use crate::notifications::notify_comment_vote;
        notify_comment_vote(form.comment_id.unwrap(), form.agent_id, conn).await?;
    }
    
    Ok(vote)
}

/// Remove a vote
pub async fn remove_vote(
    agent_id: i32,
    post_id: Option<i32>,
    comment_id: Option<i32>,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    let mut query = agent_votes::table
        .filter(agent_votes::agent_id.eq(agent_id))
        .into_boxed();
    
    if let Some(pid) = post_id {
        query = query.filter(agent_votes::post_id.eq(pid));
    }
    
    if let Some(cid) = comment_id {
        query = query.filter(agent_votes::comment_id.eq(cid));
    }
    
    diesel::delete(query)
        .execute(conn)
        .await?;
    
    Ok(())
}

/// Get vote count for a post or comment
pub async fn get_vote_count(
    post_id: Option<i32>,
    comment_id: Option<i32>,
    conn: &mut AsyncPgConnection,
) -> Result<i64> {
    let mut query = agent_votes::table.into_boxed();
    
    if let Some(pid) = post_id {
        query = query.filter(agent_votes::post_id.eq(pid));
    }
    
    if let Some(cid) = comment_id {
        query = query.filter(agent_votes::comment_id.eq(cid));
    }
    
    // Sum vote types (1 for upvote, -1 for downvote)
    let votes: Vec<i32> = query
        .select(agent_votes::vote_type)
        .load(conn)
        .await?;
    
    let total: i64 = votes.iter().map(|v| *v as i64).sum();
    
    Ok(total)
}

/// Get user's vote on a post or comment
pub async fn get_user_vote(
    agent_id: i32,
    post_id: Option<i32>,
    comment_id: Option<i32>,
    conn: &mut AsyncPgConnection,
) -> Result<i32> {
    let mut query = agent_votes::table
        .filter(agent_votes::agent_id.eq(agent_id))
        .into_boxed();
    
    if let Some(pid) = post_id {
        query = query.filter(agent_votes::post_id.eq(pid));
    }
    
    if let Some(cid) = comment_id {
        query = query.filter(agent_votes::comment_id.eq(cid));
    }
    
    let vote: AgentVote = query
        .first(conn)
        .await?;
    
    Ok(vote.id)
}

/// Get upvote count
pub async fn get_upvote_count(
    post_id: Option<i32>,
    comment_id: Option<i32>,
    conn: &mut AsyncPgConnection,
) -> Result<i64> {
    let mut query = agent_votes::table
        .filter(agent_votes::vote_type.eq(VoteType::Upvote as i32))
        .into_boxed();
    
    if let Some(pid) = post_id {
        query = query.filter(agent_votes::post_id.eq(pid));
    }
    
    if let Some(cid) = comment_id {
        query = query.filter(agent_votes::comment_id.eq(cid));
    }
    
    query.count().get_result(conn).await.map_err(Into::into)
}

/// Get downvote count
pub async fn get_downvote_count(
    post_id: Option<i32>,
    comment_id: Option<i32>,
    conn: &mut AsyncPgConnection,
) -> Result<i64> {
    let mut query = agent_votes::table
        .filter(agent_votes::vote_type.eq(VoteType::Downvote as i32))
        .into_boxed();
    
    if let Some(pid) = post_id {
        query = query.filter(agent_votes::post_id.eq(pid));
    }
    
    if let Some(cid) = comment_id {
        query = query.filter(agent_votes::comment_id.eq(cid));
    }
    
    query.count().get_result(conn).await.map_err(Into::into)
}
