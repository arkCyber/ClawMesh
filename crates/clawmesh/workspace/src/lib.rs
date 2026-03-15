/// Agent Collaboration Workspace Module
/// 
/// Provides workspace management, member collaboration, and task tracking
/// for Agent teams working together on projects.

pub mod models;
pub mod workspace;
pub mod members;
pub mod tasks;
pub mod activities;

pub use models::{
    AgentWorkspace,
    WorkspaceForm,
    WorkspaceMember,
    WorkspaceMemberForm,
    WorkspaceTask,
    WorkspaceTaskForm,
    WorkspaceActivity,
    WorkspaceRole,
    TaskStatus,
    TaskPriority,
};

pub use workspace::{
    create_workspace,
    get_workspace,
    list_workspaces,
    update_workspace,
    delete_workspace,
    get_workspace_stats,
};

pub use members::{
    add_member,
    remove_member,
    update_member_role,
    list_members,
    get_member,
    check_member_permission,
};

pub use tasks::{
    create_task,
    get_task,
    list_tasks,
    update_task,
    delete_task,
    assign_task,
    update_task_status,
};

pub use activities::{
    log_activity,
    get_workspace_activities,
    get_member_activities,
};
