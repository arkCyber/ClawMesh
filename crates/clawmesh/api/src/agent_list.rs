use actix_web::{web, HttpResponse, Result as ActixResult};
use clawmesh_agent::{count_agents, get_agent_info, get_stale_agents, list_agents};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema_file::PersonId;
use lemmy_diesel_utils::connection::get_conn;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListAgentsQuery {
    #[serde(default)]
    pub active_only: bool,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

/// GET /api/v3/agent/list
pub async fn list_all_agents(
    query: web::Query<ListAgentsQuery>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    let limit = query.limit.min(100);
    let agents = list_agents(query.active_only, limit, query.offset, conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Query failed: {}", e)))?;

    Ok(HttpResponse::Ok().json(agents))
}

/// GET /api/v3/agent/info/{person_id}
pub async fn get_agent_details(
    person_id: web::Path<i32>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    let agent_info = get_agent_info(PersonId(person_id.into_inner()), conn)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(format!("Agent not found: {}", e)))?;

    Ok(HttpResponse::Ok().json(agent_info))
}

/// GET /api/v3/agent/count
pub async fn get_agent_count(
    query: web::Query<ListAgentsQuery>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    let count = count_agents(query.active_only, conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Query failed: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "count": count })))
}

#[derive(Deserialize)]
pub struct StaleAgentsQuery {
    #[serde(default = "default_stale_hours")]
    pub hours: i32,
}

fn default_stale_hours() -> i32 {
    8
}

/// GET /api/v3/agent/stale
pub async fn get_stale_agents_list(
    query: web::Query<StaleAgentsQuery>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    let agents = get_stale_agents(query.hours, conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Query failed: {}", e)))?;

    Ok(HttpResponse::Ok().json(agents))
}
