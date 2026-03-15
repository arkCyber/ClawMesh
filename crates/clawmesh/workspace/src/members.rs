/// Workspace Member Management Functions
/// 
/// Functions for managing workspace members and permissions

use crate::models::{WorkspaceMember, WorkspaceMemberForm, WorkspaceRole, MemberWithDetails};
use anyhow::{anyhow, bail, Result};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::agent_workspace_members;

/// Add member to workspace
pub async fn add_member(
    form: WorkspaceMemberForm,
    conn: &mut AsyncPgConnection,
) -> Result<WorkspaceMember> {
    // Check if workspace exists
    use crate::workspace::workspace_exists;
    if !workspace_exists(form.workspace_id, conn).await? {
        bail!("Workspace not found");
    }
    
    // Check if agent is valid
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
    
    // Check if already a member
    let member_count: i64 = agent_workspace_members::table
        .filter(agent_workspace_members::workspace_id.eq(form.workspace_id))
        .filter(agent_workspace_members::agent_id.eq(form.agent_id))
        .count()
        .get_result(conn)
        .await?;
    
    if member_count > 0 {
        bail!("Agent is already a member");
    }
    
    // Check workspace capacity
    use lemmy_db_schema_file::schema::agent_workspaces;
    let workspace: crate::models::AgentWorkspace = agent_workspaces::table
        .find(form.workspace_id)
        .first(conn)
        .await?;
    
    let current_members: i64 = agent_workspace_members::table
        .filter(agent_workspace_members::workspace_id.eq(form.workspace_id))
        .count()
        .get_result(conn)
        .await?;
    
    if current_members >= workspace.max_members as i64 {
        bail!("Workspace is at maximum capacity");
    }
    
    // Insert member
    let member = diesel::insert_into(agent_workspace_members::table)
        .values(&form)
        .get_result::<WorkspaceMember>(conn)
        .await?;
    
    // Log activity
    use crate::activities::log_activity;
    use crate::models::{WorkspaceActivityForm, ActivityType};
    
    let activity_form = WorkspaceActivityForm {
        workspace_id: form.workspace_id,
        agent_id: form.agent_id,
        activity_type: ActivityType::MemberAdded as i32,
        target_id: Some(member.id),
        description: format!("Member added to workspace"),
        metadata: None,
    };
    
    log_activity(activity_form, conn).await?;
    
    Ok(member)
}

/// Remove member from workspace
pub async fn remove_member(
    workspace_id: i32,
    agent_id: i32,
    removed_by: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    // Check permission
    if !check_member_permission(workspace_id, removed_by, "manage_members", conn).await? {
        bail!("No permission to remove members");
    }
    
    // Cannot remove owner
    use lemmy_db_schema_file::schema::agent_workspaces;
    let workspace: crate::models::AgentWorkspace = agent_workspaces::table
        .find(workspace_id)
        .first(conn)
        .await?;
    
    if workspace.owner_id == agent_id {
        bail!("Cannot remove workspace owner");
    }
    
    // Remove member
    diesel::delete(
        agent_workspace_members::table
            .filter(agent_workspace_members::workspace_id.eq(workspace_id))
            .filter(agent_workspace_members::agent_id.eq(agent_id))
    )
    .execute(conn)
    .await?;
    
    // Log activity
    use crate::activities::log_activity;
    use crate::models::{WorkspaceActivityForm, ActivityType};
    
    let activity_form = WorkspaceActivityForm {
        workspace_id,
        agent_id: removed_by,
        activity_type: ActivityType::MemberRemoved as i32,
        target_id: Some(agent_id),
        description: format!("Member removed from workspace"),
        metadata: None,
    };
    
    log_activity(activity_form, conn).await?;
    
    Ok(())
}

/// Update member role
pub async fn update_member_role(
    workspace_id: i32,
    agent_id: i32,
    new_role: WorkspaceRole,
    updated_by: i32,
    conn: &mut AsyncPgConnection,
) -> Result<WorkspaceMember> {
    // Check permission
    if !check_member_permission(workspace_id, updated_by, "manage_members", conn).await? {
        bail!("No permission to update member roles");
    }
    
    // Cannot change owner role
    use lemmy_db_schema_file::schema::agent_workspaces;
    let workspace: crate::models::AgentWorkspace = agent_workspaces::table
        .find(workspace_id)
        .first(conn)
        .await?;
    
    if workspace.owner_id == agent_id && new_role != WorkspaceRole::Owner {
        bail!("Cannot change owner role");
    }
    
    // Update role
    let member = diesel::update(
        agent_workspace_members::table
            .filter(agent_workspace_members::workspace_id.eq(workspace_id))
            .filter(agent_workspace_members::agent_id.eq(agent_id))
    )
    .set(agent_workspace_members::role.eq(new_role as i32))
    .get_result::<WorkspaceMember>(conn)
    .await?;
    
    Ok(member)
}

/// List workspace members
pub async fn list_members(
    workspace_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<WorkspaceMember>> {
    agent_workspace_members::table
        .filter(agent_workspace_members::workspace_id.eq(workspace_id))
        .order(agent_workspace_members::joined_at.asc())
        .load::<WorkspaceMember>(conn)
        .await
        .map_err(Into::into)
}

/// Get member details
pub async fn get_member(
    workspace_id: i32,
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<WorkspaceMember> {
    agent_workspace_members::table
        .filter(agent_workspace_members::workspace_id.eq(workspace_id))
        .filter(agent_workspace_members::agent_id.eq(agent_id))
        .first::<WorkspaceMember>(conn)
        .await
        .map_err(|_| anyhow!("Member not found"))
}

/// Get member with details (including stats)
pub async fn get_member_with_details(
    workspace_id: i32,
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<MemberWithDetails> {
    let member = get_member(workspace_id, agent_id, conn).await?;
    
    // Get agent name
    use lemmy_db_schema_file::schema::person;
    let agent_name: String = person::table
        .find(agent_id)
        .select(person::name)
        .first(conn)
        .await?;
    
    // Count assigned tasks
    use lemmy_db_schema_file::schema::agent_workspace_tasks;
    let tasks_assigned: i64 = agent_workspace_tasks::table
        .filter(agent_workspace_tasks::workspace_id.eq(workspace_id))
        .filter(agent_workspace_tasks::assigned_to.eq(agent_id))
        .count()
        .get_result(conn)
        .await?;
    
    // Count completed tasks
    use crate::models::TaskStatus;
    let tasks_completed: i64 = agent_workspace_tasks::table
        .filter(agent_workspace_tasks::workspace_id.eq(workspace_id))
        .filter(agent_workspace_tasks::assigned_to.eq(agent_id))
        .filter(agent_workspace_tasks::status.eq(TaskStatus::Done as i32))
        .count()
        .get_result(conn)
        .await?;
    
    Ok(MemberWithDetails {
        member,
        agent_name,
        tasks_assigned,
        tasks_completed,
    })
}

/// Check if agent is member of workspace
pub async fn is_member(
    workspace_id: i32,
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<bool> {
    let is_member = agent_workspace_members::table
        .filter(agent_workspace_members::workspace_id.eq(workspace_id))
        .filter(agent_workspace_members::agent_id.eq(agent_id))
        .select(diesel::dsl::exists(agent_workspace_members::id))
        .get_result(conn)
        .await?;
    
    Ok(is_member)
}

/// Check member permission
pub async fn check_member_permission(
    workspace_id: i32,
    agent_id: i32,
    permission: &str,
    conn: &mut AsyncPgConnection,
) -> Result<bool> {
    let member = match get_member(workspace_id, agent_id, conn).await {
        Ok(m) => m,
        Err(_) => return Ok(false),
    };
    
    let role = match member.role {
        0 => WorkspaceRole::Owner,
        1 => WorkspaceRole::Admin,
        2 => WorkspaceRole::Member,
        3 => WorkspaceRole::Viewer,
        _ => return Ok(false),
    };
    
    let has_permission = match permission {
        "manage_members" => role.can_manage_members(),
        "create_tasks" => role.can_create_tasks(),
        "edit_workspace" => role.can_edit_workspace(),
        _ => false,
    };
    
    Ok(has_permission)
}

/// Update member last active time
pub async fn update_member_activity(
    workspace_id: i32,
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    diesel::update(
        agent_workspace_members::table
            .filter(agent_workspace_members::workspace_id.eq(workspace_id))
            .filter(agent_workspace_members::agent_id.eq(agent_id))
    )
    .set(agent_workspace_members::last_active.eq(chrono::Utc::now()))
    .execute(conn)
    .await?;
    
    Ok(())
}
