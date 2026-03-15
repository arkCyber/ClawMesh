/// Workspace Task Management Functions
/// 
/// Functions for creating, managing, and tracking tasks

use crate::models::{WorkspaceTask, WorkspaceTaskForm, TaskStatus, TaskWithDetails};
use anyhow::{anyhow, bail, Result};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::agent_workspace_tasks;

/// Create a new task
pub async fn create_task(
    form: WorkspaceTaskForm,
    conn: &mut AsyncPgConnection,
) -> Result<WorkspaceTask> {
    // Validate form
    form.validate()?;
    
    // Check permission
    use crate::members::check_member_permission;
    if !check_member_permission(form.workspace_id, form.created_by, "create_tasks", conn).await? {
        bail!("No permission to create tasks");
    }
    
    // Insert task
    let task = diesel::insert_into(agent_workspace_tasks::table)
        .values(&form)
        .get_result::<WorkspaceTask>(conn)
        .await?;
    
    // Log activity
    use crate::activities::log_activity;
    use crate::models::{WorkspaceActivityForm, ActivityType};
    
    let activity_form = WorkspaceActivityForm {
        workspace_id: form.workspace_id,
        agent_id: form.created_by,
        activity_type: ActivityType::TaskCreated as i32,
        target_id: Some(task.id),
        description: format!("Task '{}' created", task.title),
        metadata: None,
    };
    
    log_activity(activity_form, conn).await?;
    
    Ok(task)
}

/// Get task by ID
pub async fn get_task(
    task_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<WorkspaceTask> {
    agent_workspace_tasks::table
        .find(task_id)
        .first::<WorkspaceTask>(conn)
        .await
        .map_err(|_| anyhow!("Task not found"))
}

/// Get task with details
pub async fn get_task_with_details(
    task_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<TaskWithDetails> {
    let task = get_task(task_id, conn).await?;
    
    // Get assigned agent name
    use lemmy_db_schema_file::schema::person;
    let assigned_to_name = if let Some(assigned_id) = task.assigned_to {
        Some(person::table
            .find(assigned_id)
            .select(person::name)
            .first::<String>(conn)
            .await?)
    } else {
        None
    };
    
    // Get creator name
    let created_by_name: String = person::table
        .find(task.created_by)
        .select(person::name)
        .first(conn)
        .await?;
    
    Ok(TaskWithDetails {
        task,
        assigned_to_name,
        created_by_name,
    })
}

/// List tasks in workspace
pub async fn list_tasks(
    workspace_id: i32,
    status: Option<TaskStatus>,
    assigned_to: Option<i32>,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<WorkspaceTask>> {
    let mut query = agent_workspace_tasks::table
        .filter(agent_workspace_tasks::workspace_id.eq(workspace_id))
        .into_boxed();
    
    // Filter by status
    if let Some(s) = status {
        query = query.filter(agent_workspace_tasks::status.eq(s as i32));
    }
    
    // Filter by assignee
    if let Some(agent_id) = assigned_to {
        query = query.filter(agent_workspace_tasks::assigned_to.eq(agent_id));
    }
    
    // Apply pagination
    let tasks = query
        .order(agent_workspace_tasks::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<WorkspaceTask>(conn)
        .await?;
    
    Ok(tasks)
}

/// Update task
pub async fn update_task(
    task_id: i32,
    form: WorkspaceTaskForm,
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<WorkspaceTask> {
    // Validate form
    form.validate()?;
    
    // Get existing task
    let existing_task = get_task(task_id, conn).await?;
    
    // Check permission
    use crate::members::check_member_permission;
    if !check_member_permission(existing_task.workspace_id, agent_id, "create_tasks", conn).await? {
        bail!("No permission to update tasks");
    }
    
    // Update task
    let task = diesel::update(agent_workspace_tasks::table.find(task_id))
        .set(&form)
        .get_result::<WorkspaceTask>(conn)
        .await?;
    
    // Log activity
    use crate::activities::log_activity;
    use crate::models::{WorkspaceActivityForm, ActivityType};
    
    let activity_form = WorkspaceActivityForm {
        workspace_id: task.workspace_id,
        agent_id,
        activity_type: ActivityType::TaskUpdated as i32,
        target_id: Some(task.id),
        description: format!("Task '{}' updated", task.title),
        metadata: None,
    };
    
    log_activity(activity_form, conn).await?;
    
    Ok(task)
}

/// Delete task
pub async fn delete_task(
    task_id: i32,
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    // Get task
    let task = get_task(task_id, conn).await?;
    
    // Check permission (only creator or admin can delete)
    use crate::members::check_member_permission;
    let is_admin = check_member_permission(task.workspace_id, agent_id, "manage_members", conn).await?;
    
    if task.created_by != agent_id && !is_admin {
        bail!("No permission to delete task");
    }
    
    // Delete task
    diesel::delete(agent_workspace_tasks::table.find(task_id))
        .execute(conn)
        .await?;
    
    Ok(())
}

/// Assign task to agent
pub async fn assign_task(
    task_id: i32,
    assigned_to: i32,
    assigned_by: i32,
    conn: &mut AsyncPgConnection,
) -> Result<WorkspaceTask> {
    // Get task
    let task = get_task(task_id, conn).await?;
    
    // Check if assignee is member
    use crate::members::is_member;
    if !is_member(task.workspace_id, assigned_to, conn).await? {
        bail!("Assignee is not a workspace member");
    }
    
    // Check permission
    use crate::members::check_member_permission;
    if !check_member_permission(task.workspace_id, assigned_by, "create_tasks", conn).await? {
        bail!("No permission to assign tasks");
    }
    
    // Update assignment
    let updated_task = diesel::update(agent_workspace_tasks::table.find(task_id))
        .set(agent_workspace_tasks::assigned_to.eq(Some(assigned_to)))
        .get_result::<WorkspaceTask>(conn)
        .await?;
    
    // Log activity
    use crate::activities::log_activity;
    use crate::models::{WorkspaceActivityForm, ActivityType};
    
    let activity_form = WorkspaceActivityForm {
        workspace_id: task.workspace_id,
        agent_id: assigned_by,
        activity_type: ActivityType::TaskAssigned as i32,
        target_id: Some(task.id),
        description: format!("Task '{}' assigned", task.title),
        metadata: Some(serde_json::json!({
            "assigned_to": assigned_to
        })),
    };
    
    log_activity(activity_form, conn).await?;
    
    Ok(updated_task)
}

/// Update task status
pub async fn update_task_status(
    task_id: i32,
    new_status: TaskStatus,
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<WorkspaceTask> {
    // Get task
    let task = get_task(task_id, conn).await?;
    
    // Check permission
    use crate::members::is_member;
    if !is_member(task.workspace_id, agent_id, conn).await? {
        bail!("Not a workspace member");
    }
    
    // Update status and completed_at if marking as done
    let updated_task = if new_status == TaskStatus::Done {
        diesel::update(agent_workspace_tasks::table.find(task_id))
            .set((
                agent_workspace_tasks::status.eq(new_status as i32),
                agent_workspace_tasks::completed_at.eq(Some(chrono::Utc::now())),
            ))
            .get_result::<WorkspaceTask>(conn)
            .await?
    } else {
        diesel::update(agent_workspace_tasks::table.find(task_id))
            .set(agent_workspace_tasks::status.eq(new_status as i32))
            .get_result::<WorkspaceTask>(conn)
            .await?
    };
    
    // Log activity
    use crate::activities::log_activity;
    use crate::models::{WorkspaceActivityForm, ActivityType};
    
    let activity_type = if new_status == TaskStatus::Done {
        ActivityType::TaskCompleted
    } else {
        ActivityType::TaskUpdated
    };
    
    let activity_form = WorkspaceActivityForm {
        workspace_id: task.workspace_id,
        agent_id,
        activity_type: activity_type as i32,
        target_id: Some(task.id),
        description: format!("Task '{}' status changed to {:?}", task.title, new_status),
        metadata: None,
    };
    
    log_activity(activity_form, conn).await?;
    
    Ok(updated_task)
}

/// Get overdue tasks
pub async fn get_overdue_tasks(
    workspace_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<WorkspaceTask>> {
    let now = chrono::Utc::now();
    
    agent_workspace_tasks::table
        .filter(agent_workspace_tasks::workspace_id.eq(workspace_id))
        .filter(agent_workspace_tasks::due_date.lt(now))
        .filter(agent_workspace_tasks::status.ne(TaskStatus::Done as i32))
        .filter(agent_workspace_tasks::status.ne(TaskStatus::Cancelled as i32))
        .order(agent_workspace_tasks::due_date.asc())
        .load::<WorkspaceTask>(conn)
        .await
        .map_err(Into::into)
}

/// Get tasks by priority
pub async fn get_tasks_by_priority(
    workspace_id: i32,
    priority: crate::models::TaskPriority,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<WorkspaceTask>> {
    agent_workspace_tasks::table
        .filter(agent_workspace_tasks::workspace_id.eq(workspace_id))
        .filter(agent_workspace_tasks::priority.eq(priority as i32))
        .filter(agent_workspace_tasks::status.ne(TaskStatus::Done as i32))
        .filter(agent_workspace_tasks::status.ne(TaskStatus::Cancelled as i32))
        .order(agent_workspace_tasks::created_at.desc())
        .load::<WorkspaceTask>(conn)
        .await
        .map_err(Into::into)
}

// ============================================================================
// TESTS - DO-178C Level A Compliance
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{WorkspaceTaskForm, TaskStatus, TaskPriority};

    #[test]
    fn test_task_form_validation_valid() {
        let form = WorkspaceTaskForm {
            workspace_id: 1,
            title: "Test Task".to_string(),
            description: Some("A test task".to_string()),
            status: TaskStatus::Todo as i32,
            priority: TaskPriority::Medium as i32,
            assigned_to: Some(2),
            created_by: 1,
            due_date: None,
        };
        
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_task_form_validation_empty_title() {
        let form = WorkspaceTaskForm {
            workspace_id: 1,
            title: "".to_string(),
            description: None,
            status: TaskStatus::Todo as i32,
            priority: TaskPriority::Medium as i32,
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_task_form_validation_title_too_long() {
        let form = WorkspaceTaskForm {
            workspace_id: 1,
            title: "a".repeat(201),
            description: None,
            status: TaskStatus::Todo as i32,
            priority: TaskPriority::Medium as i32,
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_task_form_validation_invalid_status() {
        let form = WorkspaceTaskForm {
            workspace_id: 1,
            title: "Test".to_string(),
            description: None,
            status: -1,
            priority: TaskPriority::Medium as i32,
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        
        assert!(form.validate().is_err());

        let form2 = WorkspaceTaskForm {
            workspace_id: 1,
            title: "Test".to_string(),
            description: None,
            status: 5,
            priority: TaskPriority::Medium as i32,
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        
        assert!(form2.validate().is_err());
    }

    #[test]
    fn test_task_form_validation_invalid_priority() {
        let form = WorkspaceTaskForm {
            workspace_id: 1,
            title: "Test".to_string(),
            description: None,
            status: TaskStatus::Todo as i32,
            priority: -1,
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        
        assert!(form.validate().is_err());

        let form2 = WorkspaceTaskForm {
            workspace_id: 1,
            title: "Test".to_string(),
            description: None,
            status: TaskStatus::Todo as i32,
            priority: 4,
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        
        assert!(form2.validate().is_err());
    }

    #[test]
    fn test_task_form_validation_boundary_values() {
        // Test minimum valid title length
        let form1 = WorkspaceTaskForm {
            workspace_id: 1,
            title: "A".to_string(),
            description: None,
            status: TaskStatus::Todo as i32,
            priority: TaskPriority::Low as i32,
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        assert!(form1.validate().is_ok());

        // Test maximum valid title length
        let form2 = WorkspaceTaskForm {
            workspace_id: 1,
            title: "a".repeat(200),
            description: None,
            status: TaskStatus::Done as i32,
            priority: TaskPriority::Critical as i32,
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        assert!(form2.validate().is_ok());

        // Test maximum valid description length
        let form3 = WorkspaceTaskForm {
            workspace_id: 1,
            title: "Test".to_string(),
            description: Some("a".repeat(5000)),
            status: TaskStatus::InProgress as i32,
            priority: TaskPriority::High as i32,
            assigned_to: None,
            created_by: 1,
            due_date: None,
        };
        assert!(form3.validate().is_ok());
    }
}
