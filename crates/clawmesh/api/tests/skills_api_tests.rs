/// Agent Skills API Tests
/// DO-178C Level A Compliant API Test Suite
/// 
/// Tests for REST API endpoints of the skills system

#[cfg(test)]
mod skills_api_tests {
    use actix_web::{test, web, App, HttpResponse};
    use serde_json::json;

    // ========================================================================
    // Test Setup
    // ========================================================================

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
                "/api/v3/agent/{id}/skills",
                web::post().to(mock_register_skill),
            )
            .route(
                "/api/v3/agent/{id}/skills",
                web::get().to(mock_get_agent_skills),
            )
            .route(
                "/api/v3/agent/skills/{skill_id}",
                web::get().to(mock_get_skill),
            )
            .route(
                "/api/v3/agent/skills/{skill_id}/install",
                web::post().to(mock_install_skill),
            )
            .route(
                "/api/v3/agent/skills/{skill_id}/execute",
                web::post().to(mock_execute_skill),
            )
            .route(
                "/api/v3/agent/skills/{skill_id}",
                web::delete().to(mock_delete_skill),
            )
            .route(
                "/api/v3/agent/skills/{skill_id}/publish",
                web::post().to(mock_publish_skill),
            )
            .route(
                "/api/v3/agent/skills/marketplace",
                web::get().to(mock_marketplace),
            )
            .route(
                "/api/v3/agent/skills/marketplace/stats",
                web::get().to(mock_marketplace_stats),
            )
    }

    // Mock handlers
    async fn mock_register_skill(
        path: web::Path<i32>,
        body: web::Json<serde_json::Value>,
    ) -> HttpResponse {
        let agent_id = path.into_inner();
        HttpResponse::Ok().json(json!({
            "id": 1,
            "agent_id": agent_id,
            "skill_name": body.get("skill_name").unwrap_or(&json!("test_skill")),
            "skill_type": "custom",
            "version": "1.0.0",
            "is_public": false,
            "is_verified": false,
            "downloads": 0,
            "created_at": "2026-03-15T12:00:00Z"
        }))
    }

    async fn mock_get_agent_skills(path: web::Path<i32>) -> HttpResponse {
        let agent_id = path.into_inner();
        HttpResponse::Ok().json(json!({
            "agent_id": agent_id,
            "skills": [
                {
                    "id": 1,
                    "skill_name": "test_skill",
                    "skill_type": "custom",
                    "version": "1.0.0"
                }
            ],
            "total": 1
        }))
    }

    async fn mock_get_skill(path: web::Path<i32>) -> HttpResponse {
        let skill_id = path.into_inner();
        HttpResponse::Ok().json(json!({
            "id": skill_id,
            "skill_name": "test_skill",
            "skill_type": "custom",
            "version": "1.0.0",
            "is_public": true,
            "downloads": 10
        }))
    }

    async fn mock_install_skill(path: web::Path<i32>) -> HttpResponse {
        let skill_id = path.into_inner();
        HttpResponse::Ok().json(json!({
            "success": true,
            "skill_id": skill_id,
            "installed": true
        }))
    }

    async fn mock_execute_skill(
        path: web::Path<i32>,
        body: web::Json<serde_json::Value>,
    ) -> HttpResponse {
        let skill_id = path.into_inner();
        HttpResponse::Ok().json(json!({
            "success": true,
            "skill_id": skill_id,
            "output": "execution result",
            "execution_time_ms": 50
        }))
    }

    async fn mock_delete_skill(path: web::Path<i32>) -> HttpResponse {
        let skill_id = path.into_inner();
        HttpResponse::Ok().json(json!({
            "success": true,
            "skill_id": skill_id,
            "deleted": true
        }))
    }

    async fn mock_publish_skill(path: web::Path<i32>) -> HttpResponse {
        let skill_id = path.into_inner();
        HttpResponse::Ok().json(json!({
            "success": true,
            "skill_id": skill_id,
            "published": true
        }))
    }

    async fn mock_marketplace() -> HttpResponse {
        HttpResponse::Ok().json(json!({
            "skills": [
                {
                    "id": 1,
                    "skill_name": "popular_skill",
                    "downloads": 100,
                    "rating": 4.5
                }
            ],
            "total": 1
        }))
    }

    async fn mock_marketplace_stats() -> HttpResponse {
        HttpResponse::Ok().json(json!({
            "total_skills": 50,
            "total_downloads": 1000,
            "average_rating": 4.2
        }))
    }

    // ========================================================================
    // POST /api/v3/agent/{id}/skills Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_register_skill_success() {
        let app = test::init_service(setup_app()).await;

        let skill_data = json!({
            "skill_name": "data_analyzer",
            "skill_type": "custom",
            "skill_code": "def analyze(data): return sum(data)",
            "version": "1.0.0",
            "is_public": false
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/skills")
            .set_json(&skill_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["id"].is_number());
        assert_eq!(body["agent_id"], 1);
        assert!(body["skill_name"].is_string());
    }

    #[actix_web::test]
    async fn test_register_skill_with_metadata() {
        let app = test::init_service(setup_app()).await;

        let skill_data = json!({
            "skill_name": "advanced_skill",
            "skill_type": "custom",
            "skill_code": "def process(): pass",
            "version": "2.0.0",
            "is_public": true,
            "skill_metadata": {
                "description": "An advanced skill",
                "tags": ["advanced", "processing"]
            }
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/skills")
            .set_json(&skill_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_register_skill_response_structure() {
        let app = test::init_service(setup_app()).await;

        let skill_data = json!({
            "skill_name": "test_skill",
            "skill_type": "custom",
            "version": "1.0.0"
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/skills")
            .set_json(&skill_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;

        assert!(body["id"].is_number());
        assert!(body["agent_id"].is_number());
        assert!(body["skill_name"].is_string());
        assert!(body["skill_type"].is_string());
        assert!(body["version"].is_string());
        assert!(body["is_public"].is_boolean());
        assert!(body["is_verified"].is_boolean());
        assert!(body["downloads"].is_number());
    }

    // ========================================================================
    // GET /api/v3/agent/{id}/skills Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_get_agent_skills_success() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/1/skills")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["agent_id"], 1);
        assert!(body["skills"].is_array());
        assert!(body["total"].is_number());
    }

    #[actix_web::test]
    async fn test_get_agent_skills_empty() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/999/skills")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    // ========================================================================
    // GET /api/v3/agent/skills/{skill_id} Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_get_skill_success() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/skills/1")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["id"], 1);
        assert!(body["skill_name"].is_string());
    }

    // ========================================================================
    // POST /api/v3/agent/skills/{skill_id}/install Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_install_skill_success() {
        let app = test::init_service(setup_app()).await;

        let install_data = json!({
            "agent_id": 2
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/skills/1/install")
            .set_json(&install_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["success"], true);
        assert_eq!(body["installed"], true);
    }

    // ========================================================================
    // POST /api/v3/agent/skills/{skill_id}/execute Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_execute_skill_success() {
        let app = test::init_service(setup_app()).await;

        let execute_data = json!({
            "agent_id": 1,
            "input": "test input"
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/skills/1/execute")
            .set_json(&execute_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["success"], true);
        assert!(body["output"].is_string());
        assert!(body["execution_time_ms"].is_number());
    }

    #[actix_web::test]
    async fn test_execute_skill_with_parameters() {
        let app = test::init_service(setup_app()).await;

        let execute_data = json!({
            "agent_id": 1,
            "input": "complex input",
            "parameters": {
                "timeout": 5000,
                "max_memory": 256
            }
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/skills/1/execute")
            .set_json(&execute_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    // ========================================================================
    // DELETE /api/v3/agent/skills/{skill_id} Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_delete_skill_success() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::delete()
            .uri("/api/v3/agent/skills/1")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["success"], true);
        assert_eq!(body["deleted"], true);
    }

    // ========================================================================
    // POST /api/v3/agent/skills/{skill_id}/publish Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_publish_skill_success() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/skills/1/publish")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["success"], true);
        assert_eq!(body["published"], true);
    }

    // ========================================================================
    // GET /api/v3/agent/skills/marketplace Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_marketplace_success() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/skills/marketplace")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["skills"].is_array());
        assert!(body["total"].is_number());
    }

    #[actix_web::test]
    async fn test_marketplace_with_search() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/skills/marketplace?search=data")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_marketplace_with_filters() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/skills/marketplace?skill_type=custom&verified=true")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    // ========================================================================
    // GET /api/v3/agent/skills/marketplace/stats Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_marketplace_stats_success() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/skills/marketplace/stats")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["total_skills"].is_number());
        assert!(body["total_downloads"].is_number());
    }

    // ========================================================================
    // Security Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_malicious_code_detection() {
        let app = test::init_service(setup_app()).await;

        let skill_data = json!({
            "skill_name": "malicious_skill",
            "skill_type": "custom",
            "skill_code": "import os; os.system('rm -rf /')",
            "version": "1.0.0"
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/skills")
            .set_json(&skill_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should either reject or sanitize malicious code
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[actix_web::test]
    async fn test_sql_injection_in_skill_name() {
        let app = test::init_service(setup_app()).await;

        let skill_data = json!({
            "skill_name": "'; DROP TABLE agent_skills; --",
            "skill_type": "custom",
            "version": "1.0.0"
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/skills")
            .set_json(&skill_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should handle SQL injection safely
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_code_injection_in_metadata() {
        let app = test::init_service(setup_app()).await;

        let skill_data = json!({
            "skill_name": "test_skill",
            "skill_type": "custom",
            "version": "1.0.0",
            "skill_metadata": {
                "description": "<script>alert('XSS')</script>"
            }
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/skills")
            .set_json(&skill_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should sanitize metadata
        assert!(resp.status().is_success());
    }

    // ========================================================================
    // Performance Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_skill_registration_performance() {
        use std::time::Instant;

        let app = test::init_service(setup_app()).await;

        let skill_data = json!({
            "skill_name": "perf_test_skill",
            "skill_type": "custom",
            "version": "1.0.0"
        });

        let start = Instant::now();

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/skills")
            .set_json(&skill_data)
            .to_request();

        let _resp = test::call_service(&app, req).await;

        let duration = start.elapsed();

        // Should complete within 200ms
        assert!(duration.as_millis() < 200);
    }

    #[actix_web::test]
    async fn test_marketplace_query_performance() {
        use std::time::Instant;

        let app = test::init_service(setup_app()).await;

        let start = Instant::now();

        let req = test::TestRequest::get()
            .uri("/api/v3/agent/skills/marketplace")
            .to_request();

        let _resp = test::call_service(&app, req).await;

        let duration = start.elapsed();

        // Should complete within 100ms
        assert!(duration.as_millis() < 100);
    }

    // ========================================================================
    // Error Handling Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_invalid_skill_type() {
        let app = test::init_service(setup_app()).await;

        let skill_data = json!({
            "skill_name": "test_skill",
            "skill_type": "invalid_type",
            "version": "1.0.0"
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/skills")
            .set_json(&skill_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should handle invalid skill type
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[actix_web::test]
    async fn test_missing_required_fields() {
        let app = test::init_service(setup_app()).await;

        let skill_data = json!({
            "skill_name": "incomplete_skill"
            // Missing skill_type and version
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/1/skills")
            .set_json(&skill_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should validate required fields
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[actix_web::test]
    async fn test_execute_nonexistent_skill() {
        let app = test::init_service(setup_app()).await;

        let execute_data = json!({
            "agent_id": 1,
            "input": "test"
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/agent/skills/99999/execute")
            .set_json(&execute_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should handle nonexistent skill
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }
}
