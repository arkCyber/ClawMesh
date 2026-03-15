/// Agent Reputation API Endpoints (DO-178C Level A)
/// 
/// REST API for agent reputation management

use actix_web::{web, HttpResponse, Result as ActixResult};
use clawmesh_reputation::{
    cast_vote, get_agent_reputation, get_reputation_leaderboard,
    get_vote_history, get_vote_stats, VoteType,
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
pub struct CastVoteRequest {
    pub vote_type: String, // "upvote" or "downvote"
    pub reason: Option<String>,
}

#[derive(Serialize)]
pub struct ReputationResponse {
    pub agent_id: i32,
    pub reputation_score: i32,
    pub total_votes: i32,
    pub positive_votes: i32,
    pub negative_votes: i32,
    pub reputation_level: String,
    pub reputation_percentage: f64,
}

#[derive(Serialize)]
pub struct VoteResponse {
    pub success: bool,
    pub message: String,
    pub score_before: i32,
    pub score_after: i32,
}

#[derive(Deserialize)]
pub struct LeaderboardQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

// ============================================================================
// API Endpoints
// ============================================================================

/// GET /api/v3/agent/{person_id}/reputation
/// 
/// Get agent reputation details
pub async fn get_reputation(
    person_id: web::Path<i32>,
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

    // Get reputation
    let reputation = get_agent_reputation(pid, conn)
        .await
        .map_err(|e| {
            log::error!("Failed to get reputation: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get reputation")
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Reputation not found"))?;

    let response = ReputationResponse {
        agent_id: reputation.agent_id.0,
        reputation_score: reputation.reputation_score,
        total_votes: reputation.total_votes,
        positive_votes: reputation.positive_votes,
        negative_votes: reputation.negative_votes,
        reputation_level: reputation.reputation_level.display_name().to_string(),
        reputation_percentage: reputation.reputation_percentage(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// POST /api/v3/agent/{person_id}/reputation/vote
/// 
/// Cast a vote for an agent
pub async fn vote_for_agent(
    person_id: web::Path<i32>,
    data: web::Json<CastVoteRequest>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        log::error!("Database connection error: {}", e);
        actix_web::error::ErrorInternalServerError("Database connection failed")
    })?;

    // TODO: Get voter_id from authenticated user
    // For now, using a placeholder
    let voter_id = PersonId(1); // Replace with actual auth
    let target_id = PersonId(person_id.into_inner());

    // Parse vote type
    let vote_type = match data.vote_type.to_lowercase().as_str() {
        "upvote" => VoteType::Upvote,
        "downvote" => VoteType::Downvote,
        _ => return Err(actix_web::error::ErrorBadRequest("Invalid vote type")),
    };

    // Cast vote
    let history = cast_vote(voter_id, target_id, vote_type, data.reason.clone(), conn)
        .await
        .map_err(|e| {
            log::warn!("Vote failed: {}", e);
            actix_web::error::ErrorBadRequest(e.to_string())
        })?;

    let response = VoteResponse {
        success: true,
        message: "Vote cast successfully".to_string(),
        score_before: history.score_before,
        score_after: history.score_after,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/v3/agent/{person_id}/reputation/history
/// 
/// Get vote history for an agent
pub async fn get_reputation_history(
    person_id: web::Path<i32>,
    query: web::Query<LeaderboardQuery>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        log::error!("Database connection error: {}", e);
        actix_web::error::ErrorInternalServerError("Database connection failed")
    })?;

    let pid = PersonId(person_id.into_inner());
    let limit = query.limit.min(100);

    let history = get_vote_history(pid, limit, query.offset, conn)
        .await
        .map_err(|e| {
            log::error!("Failed to get vote history: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get history")
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "history": history,
        "limit": limit,
        "offset": query.offset
    })))
}

/// GET /api/v3/agent/reputation/leaderboard
/// 
/// Get reputation leaderboard
pub async fn get_leaderboard(
    query: web::Query<LeaderboardQuery>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        log::error!("Database connection error: {}", e);
        actix_web::error::ErrorInternalServerError("Database connection failed")
    })?;

    let limit = query.limit.min(100);

    let leaderboard = get_reputation_leaderboard(limit, query.offset, conn)
        .await
        .map_err(|e| {
            log::error!("Failed to get leaderboard: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get leaderboard")
        })?;

    let response: Vec<ReputationResponse> = leaderboard
        .into_iter()
        .map(|rep| ReputationResponse {
            agent_id: rep.agent_id.0,
            reputation_score: rep.reputation_score,
            total_votes: rep.total_votes,
            positive_votes: rep.positive_votes,
            negative_votes: rep.negative_votes,
            reputation_level: rep.reputation_level.display_name().to_string(),
            reputation_percentage: rep.reputation_percentage(),
        })
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "leaderboard": response,
        "limit": limit,
        "offset": query.offset
    })))
}

/// GET /api/v3/agent/{person_id}/reputation/stats
/// 
/// Get vote statistics for an agent
pub async fn get_reputation_stats(
    person_id: web::Path<i32>,
    context: web::Data<LemmyContext>,
) -> ActixResult<HttpResponse> {
    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await.map_err(|e| {
        log::error!("Database connection error: {}", e);
        actix_web::error::ErrorInternalServerError("Database connection failed")
    })?;

    let pid = PersonId(person_id.into_inner());

    let stats = get_vote_stats(pid, conn)
        .await
        .map_err(|e| {
            log::error!("Failed to get vote stats: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get stats")
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "stats": {
            "total_votes": stats.total_votes,
            "upvotes": stats.upvotes,
            "downvotes": stats.downvotes,
            "unique_voters": stats.unique_voters,
            "upvote_percentage": stats.upvote_percentage
        }
    })))
}
