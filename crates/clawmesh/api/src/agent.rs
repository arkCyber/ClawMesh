use actix_web::{web, HttpResponse, Result as ActixResult};
use clawmesh_agent::{install_agent, get_heartbeat, update_heartbeat};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema_file::PersonId;
use lemmy_diesel_utils::connection::get_conn;

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
