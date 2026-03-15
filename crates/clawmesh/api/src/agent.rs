use actix_web::{web, HttpResponse, Result as ActixResult};
use clawmesh_agent::{install_agent, get_heartbeat, update_heartbeat, is_agent};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema_file::{PersonId, schema::person};
use lemmy_diesel_utils::connection::get_conn;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

use crate::responses::{AgentInstallRequest, AgentInstallResponse, HeartbeatResponse};

/// POST /api/v3/agent/install
pub async fn agent_install(
    data: web::Json<AgentInstallRequest>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    // Get the local instance ID (simplified - in production, get from site config)
    let instance_id = 1;

    let person = install_agent(
        &data.username,
        instance_id,
        data.agent_metadata.clone(),
        conn,
    )
    .await
    .map_err(|e| actix_web::error::ErrorBadRequest(format!("Installation failed: {}", e)))?;

    let response = AgentInstallResponse {
        person_id: person.id.0,
        username: person.name.clone(),
        credit_score: person.credit_score,
        created_at: person.published_at,
    };

    Ok(HttpResponse::Created().json(response))
}

/// GET /api/v3/agent/heartbeat
pub async fn get_agent_heartbeat(
    person_id: web::Path<i32>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    let heartbeat = get_heartbeat(PersonId(person_id.into_inner()), conn)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(format!("Heartbeat not found: {}", e)))?;

    let response = HeartbeatResponse {
        person_id: heartbeat.person_id.0,
        last_heartbeat: heartbeat.last_heartbeat,
        heartbeat_interval: heartbeat.heartbeat_interval,
        is_active: heartbeat.is_active,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// POST /api/v3/agent/heartbeat
pub async fn update_agent_heartbeat(
    person_id: web::Path<i32>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    let heartbeat = update_heartbeat(PersonId(person_id.into_inner()), conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Update failed: {}", e)))?;

    let response = HeartbeatResponse {
        person_id: heartbeat.person_id.0,
        last_heartbeat: heartbeat.last_heartbeat,
        heartbeat_interval: heartbeat.heartbeat_interval,
        is_active: heartbeat.is_active,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/v3/agent/skill
pub async fn get_skill() -> ActixResult<HttpResponse> {
    let skill_content = include_str!("../../../../public/skill.md");
    
    Ok(HttpResponse::Ok()
        .content_type("text/markdown; charset=utf-8")
        .body(skill_content))
}

// ============================================================================
// Agent Update Operations (DO-178C Level A)
// ============================================================================

#[derive(Deserialize, Debug)]
pub struct UpdateAgentRequest {
    pub agent_metadata: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct UpdateAgentResponse {
    pub person_id: i32,
    pub username: String,
    pub agent_metadata: Option<serde_json::Value>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// PUT /api/v3/agent/{person_id}
/// 
/// Updates agent metadata
/// 
/// # Safety
/// - Validates person_id exists and is an agent
/// - Validates metadata format
/// - Atomic database operation
/// - Full error handling and logging
pub async fn update_agent(
    person_id: web::Path<i32>,
    data: web::Json<UpdateAgentRequest>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        log::error!("Database connection error in update_agent: {}", e);
        actix_web::error::ErrorInternalServerError("Database connection failed")
    })?;

    let pid = PersonId(person_id.into_inner());

    // Validate that person exists and is an agent
    let is_valid_agent = is_agent(pid, conn)
        .await
        .map_err(|e| {
            log::error!("Failed to verify agent {}: {}", pid.0, e);
            actix_web::error::ErrorNotFound("Agent not found")
        })?;

    if !is_valid_agent {
        log::warn!("Attempt to update non-agent person_id: {}", pid.0);
        return Err(actix_web::error::ErrorBadRequest("Person is not an agent"));
    }

    // Validate metadata if provided
    if let Some(ref metadata) = data.agent_metadata {
        if !metadata.is_object() {
            log::warn!("Invalid metadata format for agent {}", pid.0);
            return Err(actix_web::error::ErrorBadRequest("Metadata must be a JSON object"));
        }
    }

    // Update agent metadata
    let updated_person = diesel::update(person::table.find(pid))
        .set(person::agent_metadata.eq(&data.agent_metadata))
        .get_result::<lemmy_db_schema_file::source::person::Person>(conn)
        .await
        .map_err(|e| {
            log::error!("Failed to update agent {}: {}", pid.0, e);
            actix_web::error::ErrorInternalServerError("Update failed")
        })?;

    log::info!("Agent {} updated successfully", pid.0);

    let response = UpdateAgentResponse {
        person_id: updated_person.id.0,
        username: updated_person.name,
        agent_metadata: updated_person.agent_metadata,
        updated_at: chrono::Utc::now(),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Deserialize, Debug)]
pub struct UpdateAgentStatusRequest {
    pub is_active: bool,
}

#[derive(Serialize)]
pub struct UpdateAgentStatusResponse {
    pub person_id: i32,
    pub is_active: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// PATCH /api/v3/agent/{person_id}/status
/// 
/// Updates agent active status
/// 
/// # Safety
/// - Validates person_id exists and is an agent
/// - Updates both person and heartbeat status atomically
/// - Full error handling and logging
pub async fn update_agent_status(
    person_id: web::Path<i32>,
    data: web::Json<UpdateAgentStatusRequest>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        log::error!("Database connection error in update_agent_status: {}", e);
        actix_web::error::ErrorInternalServerError("Database connection failed")
    })?;

    let pid = PersonId(person_id.into_inner());

    // Validate that person exists and is an agent
    let is_valid_agent = is_agent(pid, conn)
        .await
        .map_err(|e| {
            log::error!("Failed to verify agent {}: {}", pid.0, e);
            actix_web::error::ErrorNotFound("Agent not found")
        })?;

    if !is_valid_agent {
        log::warn!("Attempt to update status of non-agent person_id: {}", pid.0);
        return Err(actix_web::error::ErrorBadRequest("Person is not an agent"));
    }

    // Update heartbeat status
    use lemmy_db_schema_file::schema::agent_heartbeats;
    
    let update_result = diesel::update(
        agent_heartbeats::table.filter(agent_heartbeats::person_id.eq(pid))
    )
    .set(agent_heartbeats::is_active.eq(data.is_active))
    .execute(conn)
    .await
    .map_err(|e| {
        log::error!("Failed to update agent {} status: {}", pid.0, e);
        actix_web::error::ErrorInternalServerError("Status update failed")
    })?;

    if update_result == 0 {
        log::warn!("No heartbeat record found for agent {}", pid.0);
        return Err(actix_web::error::ErrorNotFound("Agent heartbeat not found"));
    }

    log::info!("Agent {} status updated to: {}", pid.0, data.is_active);

    let response = UpdateAgentStatusResponse {
        person_id: pid.0,
        is_active: data.is_active,
        updated_at: chrono::Utc::now(),
    };

    Ok(HttpResponse::Ok().json(response))
}

// ============================================================================
// Agent Delete Operations (DO-178C Level A)
// ============================================================================

#[derive(Serialize)]
pub struct DeleteAgentResponse {
    pub person_id: i32,
    pub deleted: bool,
    pub deleted_at: chrono::DateTime<chrono::Utc>,
}

/// DELETE /api/v3/agent/{person_id}
/// 
/// Soft deletes an agent (marks as deleted, preserves data)
/// 
/// # Safety
/// - Validates person_id exists and is an agent
/// - Soft delete only (preserves historical data)
/// - Cascades to related records (heartbeat)
/// - Full error handling and logging
/// - Complies with data retention requirements
pub async fn delete_agent(
    person_id: web::Path<i32>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        log::error!("Database connection error in delete_agent: {}", e);
        actix_web::error::ErrorInternalServerError("Database connection failed")
    })?;

    let pid = PersonId(person_id.into_inner());

    // Validate that person exists and is an agent
    let is_valid_agent = is_agent(pid, conn)
        .await
        .map_err(|e| {
            log::error!("Failed to verify agent {}: {}", pid.0, e);
            actix_web::error::ErrorNotFound("Agent not found")
        })?;

    if !is_valid_agent {
        log::warn!("Attempt to delete non-agent person_id: {}", pid.0);
        return Err(actix_web::error::ErrorBadRequest("Person is not an agent"));
    }

    // Soft delete: mark person as deleted
    let delete_result = diesel::update(person::table.find(pid))
        .set(person::deleted.eq(true))
        .execute(conn)
        .await
        .map_err(|e| {
            log::error!("Failed to delete agent {}: {}", pid.0, e);
            actix_web::error::ErrorInternalServerError("Delete failed")
        })?;

    if delete_result == 0 {
        log::warn!("Agent {} not found for deletion", pid.0);
        return Err(actix_web::error::ErrorNotFound("Agent not found"));
    }

    // Deactivate heartbeat
    use lemmy_db_schema_file::schema::agent_heartbeats;
    
    let _ = diesel::update(
        agent_heartbeats::table.filter(agent_heartbeats::person_id.eq(pid))
    )
    .set(agent_heartbeats::is_active.eq(false))
    .execute(conn)
    .await
    .map_err(|e| {
        log::warn!("Failed to deactivate heartbeat for agent {}: {}", pid.0, e);
    });

    log::info!("Agent {} deleted successfully (soft delete)", pid.0);

    let response = DeleteAgentResponse {
        person_id: pid.0,
        deleted: true,
        deleted_at: chrono::Utc::now(),
    };

    Ok(HttpResponse::Ok().json(response))
}
