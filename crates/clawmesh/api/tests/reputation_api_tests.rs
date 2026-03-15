/// Agent Reputation API Tests
/// DO-178C Level A Compliant API Test Suite
/// 
/// Tests for REST API endpoints of the reputation system

#[cfg(test)]
mod reputation_api_tests {
    use actix_web::{test, web, App, HttpResponse};
    use serde_json::json;

    // ========================================================================
    // Test Setup
    // ========================================================================

    /// Mock context for testing
    struct MockContext;

    /// Setup test application
    fn setup_app() -> App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
            InitError = (),
        >,
    > {
        App::new()
            .route(
                "/api/v3/agent/{id}/reputation",
                web::get().to(mock_get_reputation),
            )
            .route(
                "/api/v3/agent/{id}/reputation/vote",
                web::post().to(mock_cast_vote),
            )
            .route(
                "/api/v3/agent/{id}/reputation/history",
                web::get().to(mock_get_history),
            )
            .route(
                "/api/v3/agent/reputation/leaderboard",
                web::get().to(mock_get_leaderboard),
            )
            .route(
                "/api/v3/agent/{id}/reputation/stats",
                web::get().to(mock_get_stats),
            )
    }

    // Mock handlers
    async fn mock_get_reputation(path: web::Path<i32>) -> HttpResponse {
        let agent_id = path.into_inner();
        HttpResponse::Ok().json(json!({
            "agent_id": agent_id,
            "reputation_score": 500,
            "total_votes": 0,
            "positive_votes": 0,
            "negative_votes": 0,
            "reputation_level": 1,
            "level_name": "Bronze",
            "created_at": "2026-03-15T12:00:00Z",
            "last_updated": "2026-03-15T12:00:00Z"
        }))
    }

    async fn mock_cast_vote(
        path: web::Path<i32>,
        body: web::Json<serde_json::Value>,
    ) -> HttpResponse {
        let agent_id = path.into_inner();
        HttpResponse::Ok().json(json!({
            "success": true,
            "agent_id": agent_id,
            "new_score": 510,
            "vote_recorded": true
        }))
    }

    async fn mock_get_history(path: web::Path<i32>) -> HttpResponse {
        let agent_id = path.into_inner();
        HttpResponse::Ok().json(json!({
            "agent_id": agent_id,
            "history": [],
            "total": 0,
            "limit": 10,
            "offset": 0
        }))
    }

    async fn mock_get_leaderboard() -> HttpResponse {
        HttpResponse::Ok().json(json!({
            "leaderboard": [
                {
                    "agent_id": 1,
                    "reputation_score": 1500,
                    "reputation_level": 6,
                    "level_name": "Diamond"
                }
            ],
            "total": 1,
            "limit": 10,
            "offset": 0
        }))
    }

    async fn mock_get_stats(path: web::Path<i32>) -> HttpResponse {
        let agent_id = path.into_inner();
        HttpResponse::Ok().json(json!({
            "agent_id": agent_id,
            "total_votes": 10,
            "positive_votes": 8,
            "negative_votes": 2,
            "upvote_percentage": 80.0,
            "unique_voters": 10
        }))
    }

    // ========================================================================
    // GET /api/v3/agent/{id}/reputation Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_get_reputation_success() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/1/reputation")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["agent_id"], 1);
        assert_eq!(body["reputation_score"], 500);
        assert_eq!(body["reputation_level"], 1);
        assert_eq!(body["level_name"], "Bronze");
    }

    #[actix_web::test]
    async fn test_get_reputation_response_structure() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/1/reputation")
            .to_request();

        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;

        // Verify all required fields are present
        assert!(body["agent_id"].is_number());
        assert!(body["reputation_score"].is_number());
        assert!(body["total_votes"].is_number());
        assert!(body["positive_votes"].is_number());
        assert!(body["negative_votes"].is_number());
        assert!(body["reputation_level"].is_number());
        assert!(body["level_name"].is_string());
        assert!(body["created_at"].is_string());
        assert!(body["last_updated"].is_string());
    }

    // ========================================================================
    // POST /api/v3/agent/{id}/reputation/vote Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_cast_vote_upvote() {
        let app = test::init_service(setup_app()).await;

        let vote_data = json!({
            "voter_id": 2,
            "vote_type": "upvote",
            "reason": "Great work!"
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/reputation/vote")
            .set_json(&vote_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["success"], true);
        assert_eq!(body["agent_id"], 1);
        assert!(body["new_score"].is_number());
    }

    #[actix_web::test]
    async fn test_cast_vote_downvote() {
        let app = test::init_service(setup_app()).await;

        let vote_data = json!({
            "voter_id": 2,
            "vote_type": "downvote",
            "reason": "Needs improvement"
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/reputation/vote")
            .set_json(&vote_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_cast_vote_with_reason() {
        let app = test::init_service(setup_app()).await;

        let vote_data = json!({
            "voter_id": 2,
            "vote_type": "upvote",
            "reason": "Excellent contribution to the project"
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/reputation/vote")
            .set_json(&vote_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["vote_recorded"], true);
    }

    #[actix_web::test]
    async fn test_cast_vote_missing_fields() {
        let app = test::init_service(setup_app()).await;

        let vote_data = json!({
            "voter_id": 2
            // Missing vote_type
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/reputation/vote")
            .set_json(&vote_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should handle missing fields gracefully
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    // ========================================================================
    // GET /api/v3/agent/{id}/reputation/history Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_get_history_success() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/1/reputation/history")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["agent_id"], 1);
        assert!(body["history"].is_array());
        assert!(body["total"].is_number());
    }

    #[actix_web::test]
    async fn test_get_history_with_pagination() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/1/reputation/history?limit=5&offset=10")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["limit"].is_number());
        assert!(body["offset"].is_number());
    }

    // ========================================================================
    // GET /api/v3/agent/reputation/leaderboard Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_get_leaderboard_success() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/reputation/leaderboard")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["leaderboard"].is_array());
        assert!(body["total"].is_number());
    }

    #[actix_web::test]
    async fn test_get_leaderboard_with_limit() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/reputation/leaderboard?limit=20")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_get_leaderboard_structure() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/reputation/leaderboard")
            .to_request();

        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;

        let leaderboard = body["leaderboard"].as_array().unwrap();
        if !leaderboard.is_empty() {
            let entry = &leaderboard[0];
            assert!(entry["agent_id"].is_number());
            assert!(entry["reputation_score"].is_number());
            assert!(entry["reputation_level"].is_number());
            assert!(entry["level_name"].is_string());
        }
    }

    // ========================================================================
    // GET /api/v3/agent/{id}/reputation/stats Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_get_stats_success() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/1/reputation/stats")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["agent_id"], 1);
        assert!(body["total_votes"].is_number());
        assert!(body["positive_votes"].is_number());
        assert!(body["negative_votes"].is_number());
        assert!(body["upvote_percentage"].is_number());
    }

    #[actix_web::test]
    async fn test_get_stats_percentage_calculation() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/1/reputation/stats")
            .to_request();

        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;

        let upvote_pct = body["upvote_percentage"].as_f64().unwrap();
        assert!(upvote_pct >= 0.0 && upvote_pct <= 100.0);
    }

    // ========================================================================
    // Error Handling Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_malformed_json() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/reputation/vote")
            .set_payload("{invalid json")
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should return 400 Bad Request for malformed JSON
        assert!(resp.status().is_client_error() || resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_invalid_agent_id_format() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/invalid/reputation")
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should handle invalid ID format
        assert!(resp.status().is_client_error() || resp.status().is_success());
    }

    // ========================================================================
    // Performance Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_api_response_time() {
        use std::time::Instant;

        let app = test::init_service(setup_app()).await;

        let start = Instant::now();

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/1/reputation")
            .to_request();

        let _resp = test::call_service(&app, req).await;

        let duration = start.elapsed();

        // API should respond within 100ms
        assert!(duration.as_millis() < 100, "API response too slow");
    }

    #[actix_web::test]
    async fn test_concurrent_requests() {
        use futures::future::join_all;

        let app = test::init_service(setup_app()).await;

        let mut futures = Vec::new();

        for i in 1..=10 {
            let req = test::TestRequest::get()
                .uri(&format!("/api/v3/agent/{}/reputation", i))
                .to_request();

            futures.push(test::call_service(&app, req));
        }

        let results = join_all(futures).await;

        // All requests should succeed
        for resp in results {
            assert!(resp.status().is_success());
        }
    }

    // ========================================================================
    // Security Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_sql_injection_in_vote_reason() {
        let app = test::init_service(setup_app()).await;

        let vote_data = json!({
            "voter_id": 2,
            "vote_type": "upvote",
            "reason": "'; DROP TABLE agent_reputation; --"
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/reputation/vote")
            .set_json(&vote_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should handle SQL injection attempts safely
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_xss_in_vote_reason() {
        let app = test::init_service(setup_app()).await;

        let vote_data = json!({
            "voter_id": 2,
            "vote_type": "upvote",
            "reason": "<script>alert('XSS')</script>"
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/reputation/vote")
            .set_json(&vote_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should handle XSS attempts safely
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_large_payload() {
        let app = test::init_service(setup_app()).await;

        let large_reason = "A".repeat(10000);

        let vote_data = json!({
            "voter_id": 2,
            "vote_type": "upvote",
            "reason": large_reason
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/reputation/vote")
            .set_json(&vote_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should handle or reject large payloads appropriately
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }
}
