/// Workspace Management Functions
/// 
/// Core functions for creating, managing, and querying workspaces

use crate::models::{AgentWorkspace, WorkspaceForm, WorkspaceStats};
use anyhow::{anyhow, bail, Result};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::agent_workspaces;

/// Create a new workspace
pub async fn create_workspace(
    form: WorkspaceForm,
    conn: &mut AsyncPgConnection,
) -> Result<AgentWorkspace> {
    // Validate form
    form.validate()?;
    
    // Check if owner is an agent
    use lemmy_db_schema_file::schema::person;
    use diesel::dsl::count;
    let owner_count: i64 = person::table
        .filter(person::id.eq(form.owner_id))
        .filter(person::user_type.eq("agent"))
        .select(count(person::id))
        .first(conn)
        .await?;
    
    if owner_count == 0 {
        bail!("Owner must be a valid agent");
    }
    
    // Insert workspace
    let workspace = diesel::insert_into(agent_workspaces::table)
        .values(&form)
        .get_result::<AgentWorkspace>(conn)
        .await?;
    
    // Add owner as first member
    use crate::members::add_member;
    use crate::models::{WorkspaceMemberForm, WorkspaceRole};
    
    let member_form = WorkspaceMemberForm {
        workspace_id: workspace.id,
        agent_id: form.owner_id,
        role: WorkspaceRole::Owner as i32,
    };
    
    add_member(member_form, conn).await?;
    
    // Log activity
    use crate::activities::log_activity;
    use crate::models::{WorkspaceActivityForm, ActivityType};
    
    let activity_form = WorkspaceActivityForm {
        workspace_id: workspace.id,
        agent_id: form.owner_id,
        activity_type: ActivityType::WorkspaceCreated as i32,
        target_id: None,
        description: format!("Workspace '{}' created", workspace.name),
        metadata: None,
    };
    
    log_activity(activity_form, conn).await?;
    
    Ok(workspace)
}

/// Get workspace by ID
pub async fn get_workspace(
    workspace_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<AgentWorkspace> {
    agent_workspaces::table
        .find(workspace_id)
        .first::<AgentWorkspace>(conn)
        .await
        .map_err(|_| anyhow!("Workspace not found"))
}

/// List workspaces
pub async fn list_workspaces(
    agent_id: Option<i32>,
    is_public: Option<bool>,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentWorkspace>> {
    let mut query = agent_workspaces::table.into_boxed();
    
    // Filter by agent membership
    if let Some(aid) = agent_id {
        use lemmy_db_schema_file::schema::agent_workspace_members;
        
        let workspace_ids: Vec<i32> = agent_workspace_members::table
            .filter(agent_workspace_members::agent_id.eq(aid))
            .select(agent_workspace_members::workspace_id)
            .load(conn)
            .await?;
        
        query = query.filter(agent_workspaces::id.eq_any(workspace_ids));
    }
    
    // Filter by public/private
    if let Some(public) = is_public {
        query = query.filter(agent_workspaces::is_public.eq(public));
    }
    
    // Apply pagination
    let workspaces = query
        .order(agent_workspaces::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<AgentWorkspace>(conn)
        .await?;
    
    Ok(workspaces)
}

/// Update workspace
pub async fn update_workspace(
    workspace_id: i32,
    form: WorkspaceForm,
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<AgentWorkspace> {
    // Validate form
    form.validate()?;
    
    // Check permission
    use crate::members::check_member_permission;
    
    if !check_member_permission(workspace_id, agent_id, "edit_workspace", conn).await? {
        bail!("No permission to edit workspace");
    }
    
    // Update workspace
    let workspace = diesel::update(agent_workspaces::table.find(workspace_id))
        .set(&form)
        .get_result::<AgentWorkspace>(conn)
        .await?;
    
    Ok(workspace)
}

/// Delete workspace
pub async fn delete_workspace(
    workspace_id: i32,
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    // Check if agent is owner
    let workspace = get_workspace(workspace_id, conn).await?;
    
    if workspace.owner_id != agent_id {
        bail!("Only owner can delete workspace");
    }
    
    // Delete all related data
    use lemmy_db_schema_file::schema::{
        agent_workspace_members,
        agent_workspace_tasks,
        agent_workspace_activities,
    };
    
    // Delete activities
    diesel::delete(
        agent_workspace_activities::table
            .filter(agent_workspace_activities::workspace_id.eq(workspace_id))
    )
    .execute(conn)
    .await?;
    
    // Delete tasks
    diesel::delete(
        agent_workspace_tasks::table
            .filter(agent_workspace_tasks::workspace_id.eq(workspace_id))
    )
    .execute(conn)
    .await?;
    
    // Delete members
    diesel::delete(
        agent_workspace_members::table
            .filter(agent_workspace_members::workspace_id.eq(workspace_id))
    )
    .execute(conn)
    .await?;
    
    // Delete workspace
    diesel::delete(agent_workspaces::table.find(workspace_id))
        .execute(conn)
        .await?;
    
    Ok(())
}

/// Get workspace statistics
pub async fn get_workspace_stats(
    workspace_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<WorkspaceStats> {
    use lemmy_db_schema_file::schema::{
        agent_workspace_members,
        agent_workspace_tasks,
        agent_workspace_activities,
    };
    use crate::models::TaskStatus;
    
    // Count members
    let total_members: i64 = agent_workspace_members::table
        .filter(agent_workspace_members::workspace_id.eq(workspace_id))
        .count()
        .get_result(conn)
        .await?;
    
    // Count total tasks
    let total_tasks: i64 = agent_workspace_tasks::table
        .filter(agent_workspace_tasks::workspace_id.eq(workspace_id))
        .count()
        .get_result(conn)
        .await?;
    
    // Count completed tasks
    let completed_tasks: i64 = agent_workspace_tasks::table
        .filter(agent_workspace_tasks::workspace_id.eq(workspace_id))
        .filter(agent_workspace_tasks::status.eq(TaskStatus::Done as i32))
        .count()
        .get_result(conn)
        .await?;
    
    // Count in-progress tasks
    let in_progress_tasks: i64 = agent_workspace_tasks::table
        .filter(agent_workspace_tasks::workspace_id.eq(workspace_id))
        .filter(agent_workspace_tasks::status.eq(TaskStatus::InProgress as i32))
        .count()
        .get_result(conn)
        .await?;
    
    // Count recent activities (last 24 hours)
    use chrono::Duration;
    let yesterday = chrono::Utc::now() - Duration::hours(24);
    
    let recent_activities: i64 = agent_workspace_activities::table
        .filter(agent_workspace_activities::workspace_id.eq(workspace_id))
        .filter(agent_workspace_activities::created_at.gt(yesterday))
        .count()
        .get_result(conn)
        .await?;
    
    Ok(WorkspaceStats {
        total_members,
        total_tasks,
        completed_tasks,
        in_progress_tasks,
        recent_activities,
    })
}

/// Check if workspace exists
pub async fn workspace_exists(
    workspace_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<bool> {
    let count: i64 = agent_workspaces::table
        .filter(agent_workspaces::id.eq(workspace_id))
        .count()
        .get_result(conn)
        .await?;
    
    Ok(count > 0)
}

/// Get workspaces owned by agent
pub async fn get_owned_workspaces(
    owner_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentWorkspace>> {
    agent_workspaces::table
        .filter(agent_workspaces::owner_id.eq(owner_id))
        .order(agent_workspaces::created_at.desc())
        .load::<AgentWorkspace>(conn)
        .await
        .map_err(Into::into)
}

// ============================================================================
// TESTS - DO-178C Level A Compliance
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::WorkspaceForm;

    // Note: These are unit tests for the business logic.
    // Integration tests with database are in tests/integration_tests.rs

    #[test]
    fn test_workspace_form_validation_valid() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "Test Workspace".to_string(),
            description: Some("A test workspace".to_string()),
            is_public: true,
            max_members: 10,
        };
        
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_workspace_form_validation_empty_name() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "".to_string(),
            description: None,
            is_public: true,
            max_members: 10,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_workspace_form_validation_name_too_long() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "a".repeat(101),
            description: None,
            is_public: true,
            max_members: 10,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_workspace_form_validation_description_too_long() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "Test".to_string(),
            description: Some("a".repeat(1001)),
            is_public: true,
            max_members: 10,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_workspace_form_validation_max_members_too_low() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "Test".to_string(),
            description: None,
            is_public: true,
            max_members: 0,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_workspace_form_validation_max_members_too_high() {
        let form = WorkspaceForm {
            owner_id: 1,
            name: "Test".to_string(),
            description: None,
            is_public: true,
            max_members: 101,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_workspace_form_validation_boundary_values() {
        // Test minimum valid name length
        let form1 = WorkspaceForm {
            owner_id: 1,
            name: "A".to_string(),
            description: None,
            is_public: true,
            max_members: 1,
        };
        assert!(form1.validate().is_ok());

        // Test maximum valid name length
        let form2 = WorkspaceForm {
            owner_id: 1,
            name: "a".repeat(100),
            description: None,
            is_public: true,
            max_members: 100,
        };
        assert!(form2.validate().is_ok());

        // Test maximum valid description length
        let form3 = WorkspaceForm {
            owner_id: 1,
            name: "Test".to_string(),
            description: Some("a".repeat(1000)),
            is_public: true,
            max_members: 50,
        };
        assert!(form3.validate().is_ok());
    }
}
