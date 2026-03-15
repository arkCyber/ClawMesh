/// Agent Workspace Data Models
/// 
/// Defines data structures for workspace collaboration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use lemmy_db_schema_file::schema::{agent_workspaces, agent_workspace_members, agent_workspace_tasks, agent_workspace_activities};

// ============================================================================
// Workspace Models
// ============================================================================

/// Agent Workspace - collaborative space for agents to work together
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = agent_workspaces)]
pub struct AgentWorkspace {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: i32, // PersonId of the owner agent
    pub is_public: bool,
    pub max_members: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Form for creating/updating workspace
#[derive(Debug, Clone, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = agent_workspaces)]
pub struct WorkspaceForm {
    pub name: String,
    pub description: Option<String>,
    pub owner_id: i32,
    pub is_public: bool,
    pub max_members: i32,
}

// ============================================================================
// Workspace Member Models
// ============================================================================

/// Workspace member role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum WorkspaceRole {
    Owner = 0,
    Admin = 1,
    Member = 2,
    Viewer = 3,
}

impl WorkspaceRole {
    pub fn can_manage_members(&self) -> bool {
        matches!(self, WorkspaceRole::Owner | WorkspaceRole::Admin)
    }

    pub fn can_create_tasks(&self) -> bool {
        matches!(self, WorkspaceRole::Owner | WorkspaceRole::Admin | WorkspaceRole::Member)
    }

    pub fn can_edit_workspace(&self) -> bool {
        matches!(self, WorkspaceRole::Owner | WorkspaceRole::Admin)
    }
}

/// Workspace member
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = agent_workspace_members)]
pub struct WorkspaceMember {
    pub id: i32,
    pub workspace_id: i32,
    pub agent_id: i32, // PersonId
    pub role: i32, // WorkspaceRole
    pub joined_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

/// Form for adding/updating workspace member
#[derive(Debug, Clone, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = agent_workspace_members)]
pub struct WorkspaceMemberForm {
    pub workspace_id: i32,
    pub agent_id: i32,
    pub role: i32,
}

// ============================================================================
// Task Models
// ============================================================================

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum TaskStatus {
    Todo = 0,
    InProgress = 1,
    Review = 2,
    Done = 3,
    Cancelled = 4,
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum TaskPriority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

/// Workspace task
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = agent_workspace_tasks)]
pub struct WorkspaceTask {
    pub id: i32,
    pub workspace_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub status: i32, // TaskStatus
    pub priority: i32, // TaskPriority
    pub assigned_to: Option<i32>, // PersonId
    pub created_by: i32, // PersonId
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Form for creating/updating task
#[derive(Debug, Clone, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = agent_workspace_tasks)]
pub struct WorkspaceTaskForm {
    pub workspace_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub status: i32,
    pub priority: i32,
    pub assigned_to: Option<i32>,
    pub created_by: i32,
    pub due_date: Option<DateTime<Utc>>,
}

// ============================================================================
// Activity Models
// ============================================================================

/// Workspace activity type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum ActivityType {
    WorkspaceCreated = 0,
    MemberAdded = 1,
    MemberRemoved = 2,
    TaskCreated = 3,
    TaskUpdated = 4,
    TaskCompleted = 5,
    TaskAssigned = 6,
    CommentAdded = 7,
}

/// Workspace activity log
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = agent_workspace_activities)]
pub struct WorkspaceActivity {
    pub id: i32,
    pub workspace_id: i32,
    pub agent_id: i32, // PersonId who performed the action
    pub activity_type: i32, // ActivityType
    pub target_id: Option<i32>, // ID of the affected entity (task, member, etc.)
    pub description: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Form for logging activity
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = agent_workspace_activities)]
pub struct WorkspaceActivityForm {
    pub workspace_id: i32,
    pub agent_id: i32,
    pub activity_type: i32,
    pub target_id: Option<i32>,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
}

// ============================================================================
// Helper Structures
// ============================================================================

/// Workspace statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceStats {
    pub total_members: i64,
    pub total_tasks: i64,
    pub completed_tasks: i64,
    pub in_progress_tasks: i64,
    pub recent_activities: i64,
}

/// Member with details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberWithDetails {
    pub member: WorkspaceMember,
    pub agent_name: String,
    pub tasks_assigned: i64,
    pub tasks_completed: i64,
}

/// Task with details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskWithDetails {
    pub task: WorkspaceTask,
    pub assigned_to_name: Option<String>,
    pub created_by_name: String,
}

// ============================================================================
// Validation
// ============================================================================

impl WorkspaceForm {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.name.is_empty() || self.name.len() > 100 {
            anyhow::bail!("Workspace name must be 1-100 characters");
        }
        
        if let Some(desc) = &self.description {
            if desc.len() > 1000 {
                anyhow::bail!("Description too long (max 1000 characters)");
            }
        }
        
        if self.max_members < 1 || self.max_members > 100 {
            anyhow::bail!("Max members must be between 1 and 100");
        }
        
        Ok(())
    }
}

impl WorkspaceTaskForm {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.title.is_empty() || self.title.len() > 200 {
            anyhow::bail!("Task title must be 1-200 characters");
        }
        
        if let Some(desc) = &self.description {
            if desc.len() > 5000 {
                anyhow::bail!("Description too long (max 5000 characters)");
            }
        }
        
        // Validate status
        if self.status < 0 || self.status > 4 {
            anyhow::bail!("Invalid task status");
        }
        
        // Validate priority
        if self.priority < 0 || self.priority > 3 {
            anyhow::bail!("Invalid task priority");
        }
        
        Ok(())
    }
}

impl WorkspaceMemberForm {
    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate role
        if self.role < 0 || self.role > 3 {
            anyhow::bail!("Invalid member role");
        }
        
        Ok(())
    }
}
