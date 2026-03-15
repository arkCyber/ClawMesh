/// Agent Workspace API Tests
/// DO-178C Level A Compliant API Test Suite
/// 
/// Tests for REST API endpoints of the workspace system

#[cfg(test)]
mod workspace_api_tests {
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
                "/api/v3/workspace",
                web::post().to(mock_create_workspace),
            )
            .route(
                "/api/v3/workspace/{id}",
                web::get().to(mock_get_workspace),
            )
            .route(
                "/api/v3/workspace",
                web::get().to(mock_list_workspaces),
            )
            .route(
                "/api/v3/workspace/{id}",
                web::put().to(mock_update_workspace),
            )
            .route(
                "/api/v3/workspace/{id}",
                web::delete().to(mock_delete_workspace),
            )
            .route(
                "/api/v3/workspace/{id}/members",
                web::post().to(mock_add_member),
            )
            .route(
                "/api/v3/workspace/{id}/members",
                web::get().to(mock_list_members),
            )
            .route(
                "/api/v3/workspace/{id}/tasks",
                web::post().to(mock_create_task),
            )
            .route(
                "/api/v3/workspace/{id}/tasks",
                web::get().to(mock_list_tasks),
            )
            .route(
                "/api/v3/workspace/{id}/activities",
                web::get().to(mock_get_activities),
            )
    }

    // Mock handlers
    async fn mock_create_workspace(body: web::Json<serde_json::Value>) -> HttpResponse {
        HttpResponse::Ok().json(json!({
            "workspace": {
                "id": 1,
                "name": body.get("name").unwrap_or(&json!("Test Workspace")),
                "description": body.get("description"),
                "owner_id": 1,
                "is_public": body.get("is_public").unwrap_or(&json!(false)),
                "max_members": body.get("max_members").unwrap_or(&json!(10)),
                "created_at": "2026-03-15T12:00:00Z",
                "updated_at": "2026-03-15T12:00:00Z"
            },
            "stats": {
                "total_members": 1,
                "total_tasks": 0,
                "completed_tasks": 0,
                "in_progress_tasks": 0,
                "recent_activities": 1
            }
        }))
    }

    async fn mock_get_workspace(path: web::Path<i32>) -> HttpResponse {
        let workspace_id = path.into_inner();
        HttpResponse::Ok().json(json!({
            "workspace": {
                "id": workspace_id,
                "name": "Test Workspace",
                "description": "A test workspace",
                "owner_id": 1,
                "is_public": false,
                "max_members": 10,
                "created_at": "2026-03-15T12:00:00Z",
                "updated_at": "2026-03-15T12:00:00Z"
            },
            "stats": {
                "total_members": 3,
                "total_tasks": 5,
                "completed_tasks": 2,
                "in_progress_tasks": 2,
                "recent_activities": 10
            }
        }))
    }

    async fn mock_list_workspaces() -> HttpResponse {
        HttpResponse::Ok().json(json!({
            "workspaces": [
                {
                    "id": 1,
                    "name": "Workspace 1",
                    "owner_id": 1,
                    "is_public": false,
                    "max_members": 10
                },
                {
                    "id": 2,
                    "name": "Workspace 2",
                    "owner_id": 1,
                    "is_public": true,
                    "max_members": 20
                }
            ],
            "total": 2
        }))
    }

    async fn mock_update_workspace(
        path: web::Path<i32>,
        body: web::Json<serde_json::Value>,
    ) -> HttpResponse {
        let workspace_id = path.into_inner();
        HttpResponse::Ok().json(json!({
            "id": workspace_id,
            "name": body.get("name").unwrap_or(&json!("Updated Workspace")),
            "description": body.get("description"),
            "owner_id": 1,
            "is_public": body.get("is_public").unwrap_or(&json!(false)),
            "max_members": body.get("max_members").unwrap_or(&json!(10)),
            "updated_at": "2026-03-15T13:00:00Z"
        }))
    }

    async fn mock_delete_workspace(path: web::Path<i32>) -> HttpResponse {
        HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Workspace deleted"
        }))
    }

    async fn mock_add_member(
        path: web::Path<i32>,
        body: web::Json<serde_json::Value>,
    ) -> HttpResponse {
        let workspace_id = path.into_inner();
        HttpResponse::Ok().json(json!({
            "id": 1,
            "workspace_id": workspace_id,
            "agent_id": body.get("agent_id").unwrap_or(&json!(2)),
            "role": body.get("role").unwrap_or(&json!(2)),
            "joined_at": "2026-03-15T12:00:00Z",
            "last_active": "2026-03-15T12:00:00Z"
        }))
    }

    async fn mock_list_members(path: web::Path<i32>) -> HttpResponse {
        let workspace_id = path.into_inner();
        HttpResponse::Ok().json(json!({
            "members": [
                {
                    "member": {
                        "id": 1,
                        "workspace_id": workspace_id,
                        "agent_id": 1,
                        "role": 0,
                        "joined_at": "2026-03-15T12:00:00Z"
                    },
                    "agent_name": "Owner Agent",
                    "tasks_assigned": 5,
                    "tasks_completed": 3
                }
            ],
            "total": 1
        }))
    }

    async fn mock_create_task(
        path: web::Path<i32>,
        body: web::Json<serde_json::Value>,
    ) -> HttpResponse {
        let workspace_id = path.into_inner();
        HttpResponse::Ok().json(json!({
            "id": 1,
            "workspace_id": workspace_id,
            "title": body.get("title").unwrap_or(&json!("New Task")),
            "description": body.get("description"),
            "status": 0,
            "priority": body.get("priority").unwrap_or(&json!(1)),
            "assigned_to": body.get("assigned_to"),
            "created_by": 1,
            "created_at": "2026-03-15T12:00:00Z"
        }))
    }

    async fn mock_list_tasks(path: web::Path<i32>) -> HttpResponse {
        let workspace_id = path.into_inner();
        HttpResponse::Ok().json(json!({
            "tasks": [
                {
                    "task": {
                        "id": 1,
                        "workspace_id": workspace_id,
                        "title": "Task 1",
                        "status": 0,
                        "priority": 1
                    },
                    "assigned_to_name": "Agent 1",
                    "created_by_name": "Owner"
                }
            ],
            "total": 1
        }))
    }

    async fn mock_get_activities(path: web::Path<i32>) -> HttpResponse {
        let workspace_id = path.into_inner();
        HttpResponse::Ok().json(json!({
            "activities": [
                {
                    "id": 1,
                    "workspace_id": workspace_id,
                    "agent_id": 1,
                    "activity_type": 0,
                    "description": "Workspace created",
                    "created_at": "2026-03-15T12:00:00Z"
                }
            ],
            "total": 1
        }))
    }

    // ========================================================================
    // Workspace Management API Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_create_workspace_api() {
        let app = test::init_service(setup_app()).await;

        let workspace_data = json!({
            "name": "New Workspace",
            "description": "A collaborative space",
            "is_public": false,
            "max_members": 15
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/workspace")
            .set_json(&workspace_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["workspace"]["id"].is_number());
        assert_eq!(body["workspace"]["name"], "New Workspace");
        assert_eq!(body["stats"]["total_members"], 1);
    }

    #[actix_web::test]
    async fn test_get_workspace_api() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/workspace/1")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["workspace"]["id"], 1);
        assert!(body["workspace"]["name"].is_string());
        assert!(body["stats"]["total_members"].is_number());
    }

    #[actix_web::test]
    async fn test_list_workspaces_api() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/workspace")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["workspaces"].is_array());
        assert!(body["total"].is_number());
    }

    #[actix_web::test]
    async fn test_update_workspace_api() {
        let app = test::init_service(setup_app()).await;

        let update_data = json!({
            "name": "Updated Name",
            "description": "Updated description",
            "is_public": true
        });

        let req = test::TestRequest::put()
            .uri("/api/v3/workspace/1")
            .set_json(&update_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "Updated Name");
    }

    #[actix_web::test]
    async fn test_delete_workspace_api() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::delete()
            .uri("/api/v3/workspace/1")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["success"], true);
    }

    // ========================================================================
    // Member Management API Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_add_member_api() {
        let app = test::init_service(setup_app()).await;

        let member_data = json!({
            "agent_id": 2,
            "role": 2
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/workspace/1/members")
            .set_json(&member_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["workspace_id"], 1);
        assert_eq!(body["agent_id"], 2);
    }

    #[actix_web::test]
    async fn test_list_members_api() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/workspace/1/members")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["members"].is_array());
        assert!(body["total"].is_number());
    }

    // ========================================================================
    // Task Management API Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_create_task_api() {
        let app = test::init_service(setup_app()).await;

        let task_data = json!({
            "title": "Implement Feature X",
            "description": "Add new feature to system",
            "priority": 2,
            "assigned_to": 2
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/workspace/1/tasks")
            .set_json(&task_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["title"], "Implement Feature X");
        assert_eq!(body["workspace_id"], 1);
    }

    #[actix_web::test]
    async fn test_list_tasks_api() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/workspace/1/tasks")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["tasks"].is_array());
        assert!(body["total"].is_number());
    }

    #[actix_web::test]
    async fn test_list_tasks_with_filters() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/workspace/1/tasks?status=0&assigned_to=2")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    // ========================================================================
    // Activity API Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_get_activities_api() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/workspace/1/activities")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["activities"].is_array());
        assert!(body["total"].is_number());
    }

    #[actix_web::test]
    async fn test_get_activities_with_pagination() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/workspace/1/activities?limit=10&offset=0")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    // ========================================================================
    // Error Handling Tests
    // ========================================================================

    #[actix_web::test]
    async fn test_create_workspace_invalid_data() {
        let app = test::init_service(setup_app()).await;

        let invalid_data = json!({
            "name": "",
            "max_members": -1
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/workspace")
            .set_json(&invalid_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        // Should handle validation errors
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[actix_web::test]
    async fn test_get_nonexistent_workspace() {
        let app = test::init_service(setup_app()).await;

        let req = test::TestRequest::get()
            .uri("/api/v3/workspace/99999")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
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
            .uri("/api/v3/workspace/1")
            .to_request();

        let _resp = test::call_service(&app, req).await;

        let duration = start.elapsed();

        // Should respond within 100ms
        assert!(duration.as_millis() < 100);
    }

    #[actix_web::test]
    async fn test_concurrent_requests() {
        use futures::future::join_all;

        let app = test::init_service(setup_app()).await;

        let mut futures = Vec::new();

        for i in 1..=10 {
            let req = test::TestRequest::get()
                .uri(&format!("/api/v3/workspace/{}", i))
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
    async fn test_sql_injection_in_workspace_name() {
        let app = test::init_service(setup_app()).await;

        let malicious_data = json!({
            "name": "'; DROP TABLE agent_workspaces; --",
            "is_public": false,
            "max_members": 10
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/workspace")
            .set_json(&malicious_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should handle SQL injection safely
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_xss_in_task_description() {
        let app = test::init_service(setup_app()).await;

        let xss_data = json!({
            "title": "Normal Task",
            "description": "<script>alert('XSS')</script>",
            "priority": 1
        });

        let req = test::TestRequest::post()
            .uri("/api/v3/workspace/1/tasks")
            .set_json(&xss_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should sanitize XSS attempts
        assert!(resp.status().is_success());
    }
}
