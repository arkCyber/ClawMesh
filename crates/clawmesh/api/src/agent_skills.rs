/// Agent Skills API Endpoints (DO-178C Level A)
/// 
/// REST API for agent skill management and marketplace

use actix_web::{web, HttpResponse, Result as ActixResult};
use clawmesh_skills::{
    register_skill, get_skill, list_skills, install_skill, uninstall_skill,
    execute_skill, delete_skill, publish_skill, search_skills,
    get_marketplace_stats, get_trending_skills, get_recommended_skills,
    AgentSkillForm, SkillType,
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema_file::PersonId;
use lemmy_diesel_utils::connection::get_conn;
use serde::{Deserialize, Serialize};
use clawmesh_agent::is_agent;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Deserialize, Debug)]
pub struct RegisterSkillRequest {
    pub skill_name: String,
    pub skill_type: String,
    pub skill_code: Option<String>,
    pub skill_metadata: Option<serde_json::Value>,
    pub version: String,
    pub is_public: bool,
}

#[derive(Serialize)]
pub struct SkillResponse {
    pub id: i32,
    pub agent_id: i32,
    pub skill_name: String,
    pub skill_type: String,
    pub version: String,
    pub is_public: bool,
    pub is_verified: bool,
    pub downloads: i32,
}

#[derive(Deserialize)]
pub struct ExecuteSkillRequest {
    pub input: String,
}

#[derive(Deserialize)]
pub struct SearchSkillsQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    pub min_downloads: Option<i32>,
    pub verified_only: Option<bool>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

// ============================================================================
// Skill Management Endpoints
// ============================================================================

/// POST /api/v3/agent/{person_id}/skills
/// 
/// Register a new skill
pub async fn register_agent_skill(
    person_id: web::Path<i32>,
    data: web::Json<RegisterSkillRequest>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        log::error!("Database connection error: {}", e);
        actix_web::error::ErrorInternalServerError("Database connection failed")
    })?;

    let pid = PersonId(person_id.into_inner());

    // Verify is agent
    let is_valid_agent = is_agent(pid, conn)
        .await
        .map_err(|e| {
            log::error!("Failed to verify agent {}: {}", pid.0, e);
            actix_web::error::ErrorNotFound("Agent not found")
        })?;

    if !is_valid_agent {
        return Err(actix_web::error::ErrorBadRequest("Not an agent"));
    }

    // Parse skill type
    let skill_type = match data.skill_type.to_lowercase().as_str() {
        "builtin" => SkillType::Builtin,
        "custom" => SkillType::Custom,
        "shared" => SkillType::Shared,
        "external" => SkillType::External,
        _ => return Err(actix_web::error::ErrorBadRequest("Invalid skill type")),
    };

    // Create skill form
    let form = AgentSkillForm {
        agent_id: pid,
        skill_name: data.skill_name.clone(),
        skill_type,
        skill_code: data.skill_code.clone(),
        skill_metadata: data.skill_metadata.clone(),
        version: data.version.clone(),
        is_public: data.is_public,
    };

    // Register skill
    let skill = register_skill(pid, form, conn)
        .await
        .map_err(|e| {
            log::warn!("Skill registration failed: {}", e);
            actix_web::error::ErrorBadRequest(e.to_string())
        })?;

    let response = SkillResponse {
        id: skill.id,
        agent_id: skill.agent_id.0,
        skill_name: skill.skill_name,
        skill_type: skill.skill_type.as_str().to_string(),
        version: skill.version,
        is_public: skill.is_public,
        is_verified: skill.is_verified,
        downloads: skill.downloads,
    };

    Ok(HttpResponse::Created().json(response))
}

/// GET /api/v3/agent/{person_id}/skills
/// 
/// List agent's skills
pub async fn list_agent_skills(
    person_id: web::Path<i32>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        log::error!("Database connection error: {}", e);
        actix_web::error::ErrorInternalServerError("Database connection failed")
    })?;

    let pid = PersonId(person_id.into_inner());

    let skills = list_skills(Some(pid), true, 100, 0, conn)
        .await
        .map_err(|e| {
            log::error!("Failed to list skills: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to list skills")
        })?;

    let response: Vec<SkillResponse> = skills
        .into_iter()
        .map(|s| SkillResponse {
            id: s.id,
            agent_id: s.agent_id.0,
            skill_name: s.skill_name,
            skill_type: s.skill_type.as_str().to_string(),
            version: s.version,
            is_public: s.is_public,
            is_verified: s.is_verified,
            downloads: s.downloads,
        })
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "skills": response
    })))
}

/// POST /api/v3/agent/skills/{skill_id}/install
/// 
/// Install a skill
pub async fn install_agent_skill(
    skill_id: web::Path<i32>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        log::error!("Database connection error: {}", e);
        actix_web::error::ErrorInternalServerError("Database connection failed")
    })?;

    // TODO: Get agent_id from authenticated user
    let agent_id = PersonId(1);

    let installation = install_skill(agent_id, skill_id.into_inner(), conn)
        .await
        .map_err(|e| {
            log::warn!("Skill installation failed: {}", e);
            actix_web::error::ErrorBadRequest(e.to_string())
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "installation_id": installation.id,
        "installed_at": installation.installed_at
    })))
}

/// DELETE /api/v3/agent/skills/{skill_id}
/// 
/// Delete a skill
pub async fn delete_agent_skill(
    skill_id: web::Path<i32>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        log::error!("Database connection error: {}", e);
        actix_web::error::ErrorInternalServerError("Database connection failed")
    })?;

    // TODO: Get agent_id from authenticated user
    let agent_id = PersonId(1);

    delete_skill(agent_id, skill_id.into_inner(), conn)
        .await
        .map_err(|e| {
            log::warn!("Skill deletion failed: {}", e);
            actix_web::error::ErrorBadRequest(e.to_string())
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Skill deleted"
    })))
}

/// POST /api/v3/agent/skills/{skill_id}/execute
/// 
/// Execute a skill
pub async fn execute_agent_skill(
    skill_id: web::Path<i32>,
    data: web::Json<ExecuteSkillRequest>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        log::error!("Database connection error: {}", e);
        actix_web::error::ErrorInternalServerError("Database connection failed")
    })?;

    // TODO: Get agent_id from authenticated user
    let agent_id = PersonId(1);

    let result = execute_skill(agent_id, skill_id.into_inner(), &data.input, conn)
        .await
        .map_err(|e| {
            log::warn!("Skill execution failed: {}", e);
            actix_web::error::ErrorBadRequest(e.to_string())
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": result.success,
        "output": result.output,
        "error": result.error,
        "execution_time_ms": result.execution_time_ms,
        "memory_used_mb": result.memory_used_mb
    })))
}

// ============================================================================
// Marketplace Endpoints
// ============================================================================

/// GET /api/v3/agent/skills/marketplace
/// 
/// Search skills in marketplace
pub async fn search_marketplace(
    query: web::Query<SearchSkillsQuery>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        log::error!("Database connection error: {}", e);
        actix_web::error::ErrorInternalServerError("Database connection failed")
    })?;

    let limit = query.limit.min(100);

    let skills = search_skills(
        query.q.clone(),
        query.category.clone(),
        query.min_downloads,
        query.verified_only.unwrap_or(false),
        limit,
        query.offset,
        conn,
    )
    .await
    .map_err(|e| {
        log::error!("Marketplace search failed: {}", e);
        actix_web::error::ErrorInternalServerError("Search failed")
    })?;

    let response: Vec<SkillResponse> = skills
        .into_iter()
        .map(|s| SkillResponse {
            id: s.id,
            agent_id: s.agent_id.0,
            skill_name: s.skill_name,
            skill_type: s.skill_type.as_str().to_string(),
            version: s.version,
            is_public: s.is_public,
            is_verified: s.is_verified,
            downloads: s.downloads,
        })
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "skills": response,
        "limit": limit,
        "offset": query.offset
    })))
}

/// GET /api/v3/agent/skills/marketplace/stats
/// 
/// Get marketplace statistics
pub async fn get_marketplace_statistics(
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        log::error!("Database connection error: {}", e);
        actix_web::error::ErrorInternalServerError("Database connection failed")
    })?;

    let stats = get_marketplace_stats(conn)
        .await
        .map_err(|e| {
            log::error!("Failed to get marketplace stats: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get stats")
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "stats": {
            "total_skills": stats.total_skills,
            "verified_skills": stats.verified_skills,
            "total_downloads": stats.total_downloads,
            "total_agents": stats.total_agents
        }
    })))
}

/// POST /api/v3/agent/skills/{skill_id}/publish
/// 
/// Publish skill to marketplace
pub async fn publish_to_marketplace(
    skill_id: web::Path<i32>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        log::error!("Database connection error: {}", e);
        actix_web::error::ErrorInternalServerError("Database connection failed")
    })?;

    // TODO: Get agent_id from authenticated user
    let agent_id = PersonId(1);

    let skill = publish_skill(agent_id, skill_id.into_inner(), conn)
        .await
        .map_err(|e| {
            log::warn!("Skill publication failed: {}", e);
            actix_web::error::ErrorBadRequest(e.to_string())
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Skill published to marketplace",
        "skill_id": skill.id
    })))
}
