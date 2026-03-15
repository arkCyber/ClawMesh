/// Agent Workspace API Endpoints
/// 
/// REST API handlers for workspace collaboration features

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use clawmesh_workspace::{
    models::*,
    workspace::*,
    members::*,
    tasks::*,
    activities::*,
};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub max_members: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub max_members: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub agent_id: i32,
    pub role: i32, // WorkspaceRole
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority: i32,
    pub assigned_to: Option<i32>,
    pub due_date: Option<String>, // ISO 8601 format
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<i32>,
    pub priority: Option<i32>,
    pub assigned_to: Option<i32>,
    pub due_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignTaskRequest {
    pub assigned_to: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskStatusRequest {
    pub status: i32,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceResponse {
    pub workspace: AgentWorkspace,
    pub stats: WorkspaceStats,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceListResponse {
    pub workspaces: Vec<AgentWorkspace>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct MemberListResponse {
    pub members: Vec<MemberWithDetails>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct TaskListResponse {
    pub tasks: Vec<TaskWithDetails>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct ActivityListResponse {
    pub activities: Vec<WorkspaceActivity>,
    pub total: i64,
}

// ============================================================================
// Workspace Management Endpoints
// ============================================================================

/// POST /api/v3/workspace
/// Create a new workspace
pub async fn create_workspace_handler(
    req: web::Json<CreateWorkspaceRequest>,
    agent_id: web::Path<i32>, // From auth middleware
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let form = WorkspaceForm {
        name: req.name.clone(),
        description: req.description.clone(),
        owner_id: *agent_id,
        is_public: req.is_public,
        max_members: req.max_members,
    };
    
    let workspace = create_workspace(form, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    let stats = get_workspace_stats(workspace.id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(WorkspaceResponse { workspace, stats }))
}

/// GET /api/v3/workspace/{id}
/// Get workspace details
pub async fn get_workspace_handler(
    workspace_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let workspace = get_workspace(*workspace_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;
    
    let stats = get_workspace_stats(*workspace_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(WorkspaceResponse { workspace, stats }))
}

/// GET /api/v3/workspace
/// List workspaces
pub async fn list_workspaces_handler(
    query: web::Query<ListQuery>,
    agent_id: Option<web::Path<i32>>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let limit = query.limit.unwrap_or(10).min(100);
    let offset = query.offset.unwrap_or(0);
    
    let workspaces = list_workspaces(
        agent_id.map(|id| *id),
        query.is_public,
        limit,
        offset,
        &mut conn,
    )
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let total = workspaces.len() as i64;
    
    Ok(HttpResponse::Ok().json(WorkspaceListResponse { workspaces, total }))
}

/// PUT /api/v3/workspace/{id}
/// Update workspace
pub async fn update_workspace_handler(
    workspace_id: web::Path<i32>,
    req: web::Json<UpdateWorkspaceRequest>,
    agent_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    // Get current workspace to preserve unchanged fields
    let current = get_workspace(*workspace_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;
    
    let form = WorkspaceForm {
        name: req.name.clone().unwrap_or(current.name),
        description: req.description.clone().or(current.description),
        owner_id: current.owner_id,
        is_public: req.is_public.unwrap_or(current.is_public),
        max_members: req.max_members.unwrap_or(current.max_members),
    };
    
    let workspace = update_workspace(*workspace_id, form, *agent_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(workspace))
}

/// DELETE /api/v3/workspace/{id}
/// Delete workspace
pub async fn delete_workspace_handler(
    workspace_id: web::Path<i32>,
    agent_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    delete_workspace(*workspace_id, *agent_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Workspace deleted"
    })))
}

// ============================================================================
// Member Management Endpoints
// ============================================================================

/// POST /api/v3/workspace/{id}/members
/// Add member to workspace
pub async fn add_member_handler(
    workspace_id: web::Path<i32>,
    req: web::Json<AddMemberRequest>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let form = WorkspaceMemberForm {
        workspace_id: *workspace_id,
        agent_id: req.agent_id,
        role: req.role,
    };
    
    let member = add_member(form, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(member))
}

/// GET /api/v3/workspace/{id}/members
/// List workspace members
pub async fn list_members_handler(
    workspace_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let members = list_members(*workspace_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    // Get details for each member
    let mut members_with_details = Vec::new();
    for member in members {
        if let Ok(details) = get_member_with_details(*workspace_id, member.agent_id, &mut conn).await {
            members_with_details.push(details);
        }
    }
    
    let total = members_with_details.len() as i64;
    
    Ok(HttpResponse::Ok().json(MemberListResponse {
        members: members_with_details,
        total,
    }))
}

/// DELETE /api/v3/workspace/{id}/members/{agent_id}
/// Remove member from workspace
pub async fn remove_member_handler(
    path: web::Path<(i32, i32)>,
    removed_by: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let (workspace_id, agent_id) = *path;
    
    remove_member(workspace_id, agent_id, *removed_by, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Member removed"
    })))
}

/// PUT /api/v3/workspace/{id}/members/{agent_id}/role
/// Update member role
pub async fn update_member_role_handler(
    path: web::Path<(i32, i32)>,
    req: web::Json<UpdateMemberRoleRequest>,
    updated_by: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let (workspace_id, agent_id) = *path;
    
    let role = match req.role {
        0 => WorkspaceRole::Owner,
        1 => WorkspaceRole::Admin,
        2 => WorkspaceRole::Member,
        3 => WorkspaceRole::Viewer,
        _ => return Err(actix_web::error::ErrorBadRequest("Invalid role")),
    };
    
    let member = update_member_role(workspace_id, agent_id, role, *updated_by, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(member))
}

// ============================================================================
// Task Management Endpoints
// ============================================================================

/// POST /api/v3/workspace/{id}/tasks
/// Create task
pub async fn create_task_handler(
    workspace_id: web::Path<i32>,
    req: web::Json<CreateTaskRequest>,
    agent_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let due_date = req.due_date.as_ref().and_then(|d| {
        chrono::DateTime::parse_from_rfc3339(d).ok().map(|dt| dt.with_timezone(&chrono::Utc))
    });
    
    let form = WorkspaceTaskForm {
        workspace_id: *workspace_id,
        title: req.title.clone(),
        description: req.description.clone(),
        status: TaskStatus::Todo as i32,
        priority: req.priority,
        assigned_to: req.assigned_to,
        created_by: *agent_id,
        due_date,
    };
    
    let task = create_task(form, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(task))
}

/// GET /api/v3/workspace/{id}/tasks
/// List tasks
pub async fn list_tasks_handler(
    workspace_id: web::Path<i32>,
    query: web::Query<TaskQuery>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let status = query.status.and_then(|s| match s {
        0 => Some(TaskStatus::Todo),
        1 => Some(TaskStatus::InProgress),
        2 => Some(TaskStatus::Review),
        3 => Some(TaskStatus::Done),
        4 => Some(TaskStatus::Cancelled),
        _ => None,
    });
    
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);
    
    let tasks = list_tasks(*workspace_id, status, query.assigned_to, limit, offset, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    // Get details for each task
    let mut tasks_with_details = Vec::new();
    for task in tasks {
        if let Ok(details) = get_task_with_details(task.id, &mut conn).await {
            tasks_with_details.push(details);
        }
    }
    
    let total = tasks_with_details.len() as i64;
    
    Ok(HttpResponse::Ok().json(TaskListResponse {
        tasks: tasks_with_details,
        total,
    }))
}

/// PUT /api/v3/workspace/tasks/{task_id}/status
/// Update task status
pub async fn update_task_status_handler(
    task_id: web::Path<i32>,
    req: web::Json<UpdateTaskStatusRequest>,
    agent_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let status = match req.status {
        0 => TaskStatus::Todo,
        1 => TaskStatus::InProgress,
        2 => TaskStatus::Review,
        3 => TaskStatus::Done,
        4 => TaskStatus::Cancelled,
        _ => return Err(actix_web::error::ErrorBadRequest("Invalid status")),
    };
    
    let task = update_task_status(*task_id, status, *agent_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(task))
}

/// POST /api/v3/workspace/tasks/{task_id}/assign
/// Assign task
pub async fn assign_task_handler(
    task_id: web::Path<i32>,
    req: web::Json<AssignTaskRequest>,
    agent_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let task = assign_task(*task_id, req.assigned_to, *agent_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(task))
}

// ============================================================================
// Activity Endpoints
// ============================================================================

/// GET /api/v3/workspace/{id}/activities
/// Get workspace activities
pub async fn get_activities_handler(
    workspace_id: web::Path<i32>,
    query: web::Query<ListQuery>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);
    
    let activities = get_workspace_activities(*workspace_id, limit, offset, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let total = activities.len() as i64;
    
    Ok(HttpResponse::Ok().json(ActivityListResponse { activities, total }))
}

// ============================================================================
// Helper Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TaskQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub status: Option<i32>,
    pub assigned_to: Option<i32>,
}

// Placeholder for DbPool type
type DbPool = deadpool::managed::Pool<diesel_async::pooled_connection::AsyncDieselConnectionManager<diesel_async::AsyncPgConnection>>;
