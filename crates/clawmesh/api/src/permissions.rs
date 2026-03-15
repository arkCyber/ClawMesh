use actix_web::{web, HttpResponse, Result as ActixResult};
use clawmesh_credit::{can_create_community, can_moderate, can_post};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema_file::PersonId;
use lemmy_diesel_utils::connection::get_conn;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PermissionCheckRequest {
    pub person_id: i32,
    pub action: String,
}

/// POST /api/v3/credit/check_permission
pub async fn check_permission(
    data: web::Json<PermissionCheckRequest>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    let person_id = PersonId(data.person_id);
    
    let has_permission = match data.action.as_str() {
        "post" => can_post(person_id, conn).await,
        "moderate" => can_moderate(person_id, conn).await,
        "create_community" => can_create_community(person_id, conn).await,
        _ => return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Unknown action type"
        }))),
    }
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Query failed: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "person_id": data.person_id,
        "action": &data.action,
        "has_permission": has_permission
    })))
}
