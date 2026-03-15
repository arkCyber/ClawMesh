/// Workspace Activity Logging Functions
/// 
/// Functions for logging and querying workspace activities

use crate::models::{WorkspaceActivity, WorkspaceActivityForm};
use anyhow::Result;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::agent_workspace_activities;

/// Log a workspace activity
pub async fn log_activity(
    form: WorkspaceActivityForm,
    conn: &mut AsyncPgConnection,
) -> Result<WorkspaceActivity> {
    let activity = diesel::insert_into(agent_workspace_activities::table)
        .values(&form)
        .get_result::<WorkspaceActivity>(conn)
        .await?;
    
    Ok(activity)
}

/// Get workspace activities
pub async fn get_workspace_activities(
    workspace_id: i32,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<WorkspaceActivity>> {
    agent_workspace_activities::table
        .filter(agent_workspace_activities::workspace_id.eq(workspace_id))
        .order(agent_workspace_activities::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<WorkspaceActivity>(conn)
        .await
        .map_err(Into::into)
}

/// Get member activities
pub async fn get_member_activities(
    workspace_id: i32,
    agent_id: i32,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<WorkspaceActivity>> {
    agent_workspace_activities::table
        .filter(agent_workspace_activities::workspace_id.eq(workspace_id))
        .filter(agent_workspace_activities::agent_id.eq(agent_id))
        .order(agent_workspace_activities::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<WorkspaceActivity>(conn)
        .await
        .map_err(Into::into)
}

/// Get recent activities (last 24 hours)
pub async fn get_recent_activities(
    workspace_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<WorkspaceActivity>> {
    use chrono::Duration;
    let yesterday = chrono::Utc::now() - Duration::hours(24);
    
    agent_workspace_activities::table
        .filter(agent_workspace_activities::workspace_id.eq(workspace_id))
        .filter(agent_workspace_activities::created_at.gt(yesterday))
        .order(agent_workspace_activities::created_at.desc())
        .load::<WorkspaceActivity>(conn)
        .await
        .map_err(Into::into)
}

/// Get activities by type
pub async fn get_activities_by_type(
    workspace_id: i32,
    activity_type: crate::models::ActivityType,
    limit: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<WorkspaceActivity>> {
    agent_workspace_activities::table
        .filter(agent_workspace_activities::workspace_id.eq(workspace_id))
        .filter(agent_workspace_activities::activity_type.eq(activity_type as i32))
        .order(agent_workspace_activities::created_at.desc())
        .limit(limit)
        .load::<WorkspaceActivity>(conn)
        .await
        .map_err(Into::into)
}
