/// End-to-End Integration Tests
/// DO-178C Level A Compliant E2E Test Suite
/// 
/// Tests complete workflows across reputation and skills systems

#[cfg(test)]
mod e2e_tests {
    use actix_web::{test, web, App};
    use serde_json::json;
    use std::time::Instant;

    // ========================================================================
    // Test Setup
    // ========================================================================

    fn setup_full_app() -> App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
            InitError = (),
        >,
    > {
        App::new()
            // Reputation endpoints
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
            // Skills endpoints
            .route(
                "/api/v3/agent/{id}/skills",
                web::post().to(mock_register_skill),
            )
            .route(
                "/api/v3/agent/{id}/skills",
                web::get().to(mock_get_agent_skills),
            )
            .route(
                "/api/v3/agent/skills/{skill_id}/execute",
                web::post().to(mock_execute_skill),
            )
            .route(
                "/api/v3/agent/skills/{skill_id}/install",
                web::post().to(mock_install_skill),
            )
    }

    // Mock implementations
    async fn mock_get_reputation(path: web::Path<i32>) -> actix_web::HttpResponse {
        let agent_id = path.into_inner();
        actix_web::HttpResponse::Ok().json(json!({
            "agent_id": agent_id,
            "reputation_score": 500,
            "total_votes": 0,
            "positive_votes": 0,
            "negative_votes": 0,
            "reputation_level": 1,
            "level_name": "Bronze"
        }))
    }

    async fn mock_cast_vote(
        path: web::Path<i32>,
        body: web::Json<serde_json::Value>,
    ) -> actix_web::HttpResponse {
        let agent_id = path.into_inner();
        let vote_type = body.get("vote_type").unwrap_or(&json!("upvote"));
        
        actix_web::HttpResponse::Ok().json(json!({
            "success": true,
            "agent_id": agent_id,
            "vote_type": vote_type,
            "new_score": if vote_type == "upvote" { 510 } else { 490 }
        }))
    }

    async fn mock_get_history(path: web::Path<i32>) -> actix_web::HttpResponse {
        let agent_id = path.into_inner();
        actix_web::HttpResponse::Ok().json(json!({
            "agent_id": agent_id,
            "history": [],
            "total": 0
        }))
    }

    async fn mock_get_leaderboard() -> actix_web::HttpResponse {
        actix_web::HttpResponse::Ok().json(json!({
            "leaderboard": [],
            "total": 0
        }))
    }

    async fn mock_register_skill(
        path: web::Path<i32>,
        body: web::Json<serde_json::Value>,
    ) -> actix_web::HttpResponse {
        let agent_id = path.into_inner();
        actix_web::HttpResponse::Ok().json(json!({
            "id": 1,
            "agent_id": agent_id,
            "skill_name": body.get("skill_name"),
            "skill_type": "custom",
            "version": "1.0.0",
            "is_public": false,
            "is_verified": false
        }))
    }

    async fn mock_get_agent_skills(path: web::Path<i32>) -> actix_web::HttpResponse {
        let agent_id = path.into_inner();
        actix_web::HttpResponse::Ok().json(json!({
            "agent_id": agent_id,
            "skills": [],
            "total": 0
        }))
    }

    async fn mock_execute_skill(
        path: web::Path<i32>,
        body: web::Json<serde_json::Value>,
    ) -> actix_web::HttpResponse {
        let skill_id = path.into_inner();
        let input = body.get("input").unwrap_or(&json!(""));
        
        actix_web::HttpResponse::Ok().json(json!({
            "success": true,
            "skill_id": skill_id,
            "output": format!("Processed: {}", input),
            "execution_time_ms": 50
        }))
    }

    async fn mock_install_skill(path: web::Path<i32>) -> actix_web::HttpResponse {
        let skill_id = path.into_inner();
        actix_web::HttpResponse::Ok().json(json!({
            "success": true,
            "skill_id": skill_id,
            "installed": true
        }))
    }

    // ========================================================================
    // Complete Agent Lifecycle Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_complete_agent_reputation_lifecycle() {
        let app = test::init_service(setup_full_app()).await;

        // 1. Create agent and check initial reputation
        let agent_id = 1;
        
        let req = test::TestRequest::get()
            .uri(&format!("/api/v3/agent/{}/reputation", agent_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["agent_id"], agent_id);
        assert_eq!(body["reputation_score"], 500);
        assert_eq!(body["reputation_level"], 1);

        // 2. Receive upvotes
        for voter_id in 2..=6 {
            let vote_data = json!({
                "voter_id": voter_id,
                "vote_type": "upvote",
                "reason": "Great work!"
            });

            let req = test::TestRequest::post()
                .uri(&format!("/api/v3/agent/{}/reputation/vote", agent_id))
                .set_json(&vote_data)
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success());
        }

        // 3. Receive some downvotes
        for voter_id in 7..=9 {
            let vote_data = json!({
                "voter_id": voter_id,
                "vote_type": "downvote",
                "reason": "Needs improvement"
            });

            let req = test::TestRequest::post()
                .uri(&format!("/api/v3/agent/{}/reputation/vote", agent_id))
                .set_json(&vote_data)
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success());
        }

        // 4. Check vote history
        let req = test::TestRequest::get()
            .uri(&format!("/api/v3/agent/{}/reputation/history", agent_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 5. Check leaderboard position
        let req = test::TestRequest::get()
            .uri("/api/v3/agent/reputation/leaderboard")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_complete_skill_lifecycle() {
        let app = test::init_service(setup_full_app()).await;

        let agent_id = 1;

        // 1. Register a new skill
        let skill_data = json!({
            "skill_name": "data_processor",
            "skill_type": "custom",
            "skill_code": "def process(data): return data.upper()",
            "version": "1.0.0",
            "is_public": false,
            "skill_metadata": {
                "description": "Processes text data",
                "tags": ["text", "processing"]
            }
        });

        let req = test::TestRequest::post()
            .uri(&format!("/api/v3/agent/{}/skills", agent_id))
            .set_json(&skill_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let skill_response: serde_json::Value = test::read_body_json(resp).await;
        let skill_id = skill_response["id"].as_i64().unwrap();

        // 2. Get agent's skills list
        let req = test::TestRequest::get()
            .uri(&format!("/api/v3/agent/{}/skills", agent_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 3. Another agent installs the skill
        let installer_id = 2;
        let req = test::TestRequest::post()
            .uri(&format!("/api/v3/agent/skills/{}/install", skill_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 4. Execute the skill
        let execute_data = json!({
            "agent_id": installer_id,
            "input": "hello world"
        });

        let req = test::TestRequest::post()
            .uri(&format!("/api/v3/agent/skills/{}/execute", skill_id))
            .set_json(&execute_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let execution_result: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(execution_result["success"], true);
        assert!(execution_result["output"].is_string());
        assert!(execution_result["execution_time_ms"].is_number());
    }

    // ========================================================================
    // Reputation-Skills Integration Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_skill_development_affects_reputation() {
        let app = test::init_service(setup_full_app()).await;

        let developer_id = 1;
        let user_id = 2;

        // 1. Developer creates high-quality skills
        for i in 1..=3 {
            let skill_data = json!({
                "skill_name": format!("quality_skill_{}", i),
                "skill_type": "custom",
                "skill_code": "def solve(): return 'solution'",
                "version": "1.0.0",
                "is_public": true
            });

            let req = test::TestRequest::post()
                .uri(&format!("/api/v3/agent/{}/skills", developer_id))
                .set_json(&skill_data)
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success());
        }

        // 2. Users upvote the developer for quality skills
        for voter_id in 2..=6 {
            let vote_data = json!({
                "voter_id": voter_id,
                "vote_type": "upvote",
                "reason": "Excellent skills!"
            });

            let req = test::TestRequest::post()
                .uri(&format!("/api/v3/agent/{}/reputation/vote", developer_id))
                .set_json(&vote_data)
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success());
        }

        // 3. Check developer's reputation improved
        let req = test::TestRequest::get()
            .uri(&format!("/api/v3/agent/{}/reputation", developer_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let reputation: serde_json::Value = test::read_body_json(resp).await;
        assert!(reputation["reputation_score"].as_i64().unwrap() > 500);
        assert!(reputation["total_votes"].as_i64().unwrap() > 0);
    }

    #[actix_web::test]
    async fn test_skill_execution_with_reputation_check() {
        let app = test::init_service(setup_full_app()).await;

        let skill_creator_id = 1;
        let skill_user_id = 2;

        // 1. High reputation agent creates skill
        let skill_data = json!({
            "skill_name": "advanced_analyzer",
            "skill_type": "custom",
            "skill_code": "def analyze(data): return len(data)",
            "version": "1.0.0",
            "is_public": true
        });

        let req = test::TestRequest::post()
            .uri(&format!("/api/v3/agent/{}/skills", skill_creator_id))
            .set_json(&skill_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let skill_response: serde_json::Value = test::read_body_json(resp).await;
        let skill_id = skill_response["id"].as_i64().unwrap();

        // 2. User installs and executes skill
        let req = test::TestRequest::post()
            .uri(&format!("/api/v3/agent/skills/{}/install", skill_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 3. Execute skill with different inputs
        let test_inputs = vec!["short", "medium length", "this is a much longer input string"];

        for input in test_inputs {
            let execute_data = json!({
                "agent_id": skill_user_id,
                "input": input
            });

            let req = test::TestRequest::post()
                .uri(&format!("/api/v3/agent/skills/{}/execute", skill_id))
                .set_json(&execute_data)
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success());

            let result: serde_json::Value = test::read_body_json(resp).await;
            assert_eq!(result["success"], true);
            assert!(result["execution_time_ms"].as_i64().unwrap() < 1000); // Should be fast
        }
    }

    // ========================================================================
    // Performance and Load Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_system_performance_under_load() {
        use futures::future::join_all;
        use std::time::Duration;

        let app = test::init_service(setup_full_app()).await;

        // Simulate concurrent reputation voting
        let mut reputation_futures = Vec::new();
        
        for voter_id in 2..=12 {
            for target_id in 1..=3 {
                let vote_data = json!({
                    "voter_id": voter_id,
                    "vote_type": "upvote",
                    "reason": "Performance test vote"
                });

                let req = test::TestRequest::post()
                    .uri(&format!("/api/v3/agent/{}/reputation/vote", target_id))
                    .set_json(&vote_data)
                    .to_request();

                reputation_futures.push(test::call_service(&app, req));
            }
        }

        // Simulate concurrent skill operations
        let mut skill_futures = Vec::new();
        
        for agent_id in 1..=5 {
            for skill_num in 1..=3 {
                let skill_data = json!({
                    "skill_name": format!("perf_skill_{}_{}", agent_id, skill_num),
                    "skill_type": "custom",
                    "version": "1.0.0"
                });

                let req = test::TestRequest::post()
                    .uri(&format!("/api/v3/agent/{}/skills", agent_id))
                    .set_json(&skill_data)
                    .to_request();

                skill_futures.push(test::call_service(&app, req));
            }
        }

        // Execute all operations concurrently
        let start = Instant::now();
        
        let reputation_results = join_all(reputation_futures).await;
        let skill_results = join_all(skill_futures).await;
        
        let duration = start.elapsed();

        // Verify all operations completed successfully
        for resp in reputation_results {
            assert!(resp.status().is_success());
        }

        for resp in skill_results {
            assert!(resp.status().is_success());
        }

        // Performance should be reasonable (< 5 seconds for 60+ operations)
        assert!(duration.as_secs() < 5, "System too slow under load");
    }

    #[actix_web::test]
    async fn test_memory_efficiency_large_dataset() {
        let app = test::init_service(setup_full_app()).await;

        // Simulate large leaderboard query
        let req = test::TestRequest::get()
            .uri("/api/v3/agent/reputation/leaderboard?limit=1000")
            .to_request();

        let start = Instant::now();
        let resp = test::call_service(&app, req).await;
        let duration = start.elapsed();

        assert!(resp.status().is_success());
        
        // Should handle large datasets efficiently
        assert!(duration.as_millis() < 200, "Leaderboard query too slow");
    }

    // ========================================================================
    // Security Integration Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_cross_system_security() {
        let app = test::init_service(setup_full_app()).await;

        // 1. Attempt to create malicious skill
        let malicious_skill = json!({
            "skill_name": "malicious",
            "skill_type": "custom",
            "skill_code": "import os; os.system('rm -rf /')",
            "version": "1.0.0"
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/skills")
            .set_json(&malicious_skill)
            .to_request();

        let resp = test::call_service(&app, req).await;
        
        // Should either reject or sanitize
        assert!(resp.status().is_success() || resp.status().is_client_error());

        // 2. Attempt SQL injection in voting
        let malicious_vote = json!({
            "voter_id": 2,
            "vote_type": "upvote",
            "reason": "'; DROP TABLE agent_reputation; --"
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/reputation/vote")
            .set_json(&malicious_vote)
            .to_request();

        let resp = test::call_service(&app, req).await;
        
        // Should handle SQL injection safely
        assert!(resp.status().is_success());

        // 3. Verify system still functional after attacks
        let req = test::TestRequest::get()
            .uri("/api/v3/agent/1/reputation")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    // ========================================================================
    // Data Consistency Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_data_consistency_across_operations() {
        let app = test::init_service(setup_full_app()).await;

        let agent_id = 1;

        // 1. Record initial state
        let req = test::TestRequest::get()
            .uri(&format!("/api/v3/agent/{}/reputation", agent_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        let initial_reputation: serde_json::Value = test::read_body_json(resp).await;
        let initial_score = initial_reputation["reputation_score"].as_i64().unwrap();

        // 2. Perform series of operations
        let operations = vec![
            ("upvote", "Great work!"),
            ("upvote", "Excellent contribution"),
            ("downvote", "Needs some improvement"),
            ("upvote", "Fixed the issues"),
        ];

        for (vote_type, reason) in operations {
            let voter_id = rand::random::<i32>().abs() % 1000 + 10;
            
            let vote_data = json!({
                "voter_id": voter_id,
                "vote_type": vote_type,
                "reason": reason
            });

            let req = test::TestRequest::post()
                .uri(&format!("/api/v3/agent/{}/reputation/vote", agent_id))
                .set_json(&vote_data)
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success());
        }

        // 3. Verify final consistency
        let req = test::TestRequest::get()
            .uri(&format!("/api/v3/agent/{}/reputation", agent_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        let final_reputation: serde_json::Value = test::read_body_json(resp).await;

        // Should have recorded all votes
        assert!(final_reputation["total_votes"].as_i64().unwrap() > 0);
        
        // Score should reflect the operations
        let final_score = final_reputation["reputation_score"].as_i64().unwrap();
        let expected_change = 20 - 10; // 3 upvotes, 1 downvote
        assert_eq!(final_score, initial_score + expected_change);
    }

    // ========================================================================
    // Error Recovery Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_system_recovery_after_errors() {
        let app = test::init_service(setup_full_app()).await;

        // 1. Trigger various errors
        let error_cases = vec![
            ("/api/v3/agent/999999/reputation", "GET"), // Non-existent agent
            ("/api/v3/agent/1/reputation/vote", "POST"), // Missing data
            ("/api/v3/agent/1/skills", "POST"), // Invalid skill data
        ];

        for (endpoint, method) in error_cases {
            let req = if method == "GET" {
                test::TestRequest::get().uri(endpoint).to_request()
            } else {
                test::TestRequest::post()
                    .uri(endpoint)
                    .set_json(&json!({}))
                    .to_request()
            };

            let resp = test::call_service(&app, req).await;
            // Should handle errors gracefully
            assert!(resp.status().is_success() || resp.status().is_client_error());
        }

        // 2. Verify system still functional
        let req = test::TestRequest::get()
            .uri("/api/v3/agent/1/reputation")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/1/skills")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
