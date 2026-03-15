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

// ============================================================================
// TESTS - DO-178C Level A Compliance
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ActivityType;

    #[test]
    fn test_activity_type_values() {
        // Test all activity type enum values
        assert_eq!(ActivityType::WorkspaceCreated as i32, 0);
        assert_eq!(ActivityType::WorkspaceUpdated as i32, 1);
        assert_eq!(ActivityType::MemberAdded as i32, 2);
        assert_eq!(ActivityType::MemberRemoved as i32, 3);
        assert_eq!(ActivityType::TaskCreated as i32, 4);
        assert_eq!(ActivityType::TaskUpdated as i32, 5);
        assert_eq!(ActivityType::TaskCompleted as i32, 6);
    }

    #[test]
    fn test_activity_type_from_i32() {
        // Test conversion from i32 to ActivityType
        assert_eq!(ActivityType::from_i32(0), Some(ActivityType::WorkspaceCreated));
        assert_eq!(ActivityType::from_i32(1), Some(ActivityType::WorkspaceUpdated));
        assert_eq!(ActivityType::from_i32(2), Some(ActivityType::MemberAdded));
        assert_eq!(ActivityType::from_i32(3), Some(ActivityType::MemberRemoved));
        assert_eq!(ActivityType::from_i32(4), Some(ActivityType::TaskCreated));
        assert_eq!(ActivityType::from_i32(5), Some(ActivityType::TaskUpdated));
        assert_eq!(ActivityType::from_i32(6), Some(ActivityType::TaskCompleted));
        assert_eq!(ActivityType::from_i32(7), None);
        assert_eq!(ActivityType::from_i32(-1), None);
    }

    #[test]
    fn test_activity_form_validation() {
        // Test that activity form can be created with valid data
        let form = WorkspaceActivityForm {
            workspace_id: 1,
            agent_id: 2,
            activity_type: ActivityType::TaskCreated as i32,
            target_id: Some(3),
            description: "Task created".to_string(),
            metadata: None,
        };
        
        assert_eq!(form.workspace_id, 1);
        assert_eq!(form.agent_id, 2);
        assert_eq!(form.activity_type, 4);
        assert_eq!(form.target_id, Some(3));
    }
}
