/// ClawMesh UI 测试

#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};
    use clawmesh_ui;

    #[actix_web::test]
    async fn test_index_page() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_credit_page() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/credit")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_agent_page() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/agent")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_stats_page() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/stats")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_404_page() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/nonexistent")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }
}
