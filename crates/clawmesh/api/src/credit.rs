use actix_web::{web, HttpResponse, Result as ActixResult};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::person::Person;
use lemmy_db_schema_file::PersonId;
use lemmy_diesel_utils::connection::get_conn;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::responses::{CreditResponse, CreditHistoryResponse, CreditHistoryItem};

/// GET /api/v3/user/{id}/credit
pub async fn get_user_credit(
    person_id: web::Path<i32>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    use lemmy_db_schema_file::schema::person;

    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    let person = person::table
        .find(PersonId(person_id.into_inner()))
        .first::<Person>(conn)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(format!("User not found: {}", e)))?;

    let response = CreditResponse {
        person_id: person.id.0,
        username: person.name.clone(),
        credit_score: person.credit_score,
        reputation_tier: person.reputation_tier.clone(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/v3/user/{id}/credit/history
pub async fn get_credit_history(
    person_id: web::Path<i32>,
    query: web::Query<std::collections::HashMap<String, String>>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    use lemmy_db_schema_file::schema::person;

    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    let pid = PersonId(person_id.into_inner());

    let person = person::table
        .find(pid)
        .first::<Person>(conn)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(format!("User not found: {}", e)))?;

    let limit = query
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(50)
        .min(100);

    let history = clawmesh_credit::get_credit_history(pid, limit, conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Query failed: {}", e)))?;

    let history_items: Vec<CreditHistoryItem> = history
        .into_iter()
        .map(|h| CreditHistoryItem {
            action_type: h.action_type,
            credit_change: h.credit_change,
            reason: h.reason,
            created_at: h.created_at,
        })
        .collect();

    let response = CreditHistoryResponse {
        person_id: person.id.0,
        username: person.name.clone(),
        credit_score: person.credit_score,
        reputation_tier: person.reputation_tier.clone(),
        history: history_items,
    };

    Ok(HttpResponse::Ok().json(response))
}
