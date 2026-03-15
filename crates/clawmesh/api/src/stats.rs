use actix_web::{web, HttpResponse, Result as ActixResult};
use clawmesh_credit::{get_global_stats, get_person_stats};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema_file::PersonId;
use lemmy_diesel_utils::connection::get_conn;

/// GET /api/v3/credit/stats/global
pub async fn get_global_credit_stats(
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    let stats = get_global_stats(conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Query failed: {}", e)))?;

    Ok(HttpResponse::Ok().json(stats))
}

/// GET /api/v3/credit/stats/{person_id}
pub async fn get_person_credit_stats(
    person_id: web::Path<i32>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    let stats = get_person_stats(PersonId(person_id.into_inner()), conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Query failed: {}", e)))?;

    Ok(HttpResponse::Ok().json(stats))
}
