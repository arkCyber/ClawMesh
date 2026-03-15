/// ClawMesh UI 性能测试
/// 
/// 测试页面加载时间、并发处理能力等性能指标

use actix_web::{test, web, App};
use clawmesh_ui;
use std::time::Instant;

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn test_index_page_load_time() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        let start = Instant::now();
        
        let req = test::TestRequest::get()
            .uri("/clawmesh/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        
        let duration = start.elapsed();
        
        assert!(resp.status().is_success());
        // 页面加载时间应该小于 200ms
        assert!(duration.as_millis() < 200, "Page load time: {:?}", duration);
    }

    #[actix_web::test]
    async fn test_credit_page_load_time() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        let start = Instant::now();
        
        let req = test::TestRequest::get()
            .uri("/clawmesh/credit")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        
        let duration = start.elapsed();
        
        assert!(resp.status().is_success());
        assert!(duration.as_millis() < 200, "Page load time: {:?}", duration);
    }

    #[actix_web::test]
    async fn test_agent_page_load_time() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        let start = Instant::now();
        
        let req = test::TestRequest::get()
            .uri("/clawmesh/agent")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        
        let duration = start.elapsed();
        
        assert!(resp.status().is_success());
        assert!(duration.as_millis() < 200, "Page load time: {:?}", duration);
    }

    #[actix_web::test]
    async fn test_stats_page_load_time() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        let start = Instant::now();
        
        let req = test::TestRequest::get()
            .uri("/clawmesh/stats")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        
        let duration = start.elapsed();
        
        assert!(resp.status().is_success());
        assert!(duration.as_millis() < 200, "Page load time: {:?}", duration);
    }

    #[actix_web::test]
    async fn test_concurrent_requests() {
        use tokio::task;
        
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        let start = Instant::now();
        
        // 创建 100 个并发请求
        let mut handles = vec![];
        
        for _ in 0..100 {
            let app_clone = app.clone();
            let handle = task::spawn(async move {
                let req = test::TestRequest::get()
                    .uri("/clawmesh/")
                    .to_request();
                
                test::call_service(&app_clone, req).await
            });
            handles.push(handle);
        }
        
        // 等待所有请求完成
        for handle in handles {
            let resp = handle.await.unwrap();
            assert!(resp.status().is_success());
        }
        
        let duration = start.elapsed();
        
        // 100 个并发请求应该在 2 秒内完成
        assert!(duration.as_secs() < 2, "Concurrent requests took: {:?}", duration);
    }

    #[actix_web::test]
    async fn test_template_render_performance() {
        use clawmesh_ui::templates::*;
        use askama::Template;
        
        let start = Instant::now();
        
        // 渲染 1000 次模板
        for _ in 0..1000 {
            let template = IndexTemplate {
                title: "Performance Test".to_string(),
            };
            
            let _html = template.render().expect("Failed to render");
        }
        
        let duration = start.elapsed();
        
        // 1000 次渲染应该在 100ms 内完成
        assert!(duration.as_millis() < 100, "Template rendering took: {:?}", duration);
    }

    #[actix_web::test]
    async fn test_response_size() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        
        let size_kb = body.len() / 1024;
        
        // 页面大小应该小于 100KB
        assert!(size_kb < 100, "Page size: {} KB", size_kb);
    }

    #[actix_web::test]
    async fn test_memory_usage() {
        use clawmesh_ui::templates::*;
        use askama::Template;
        
        // 创建大量模板实例测试内存使用
        let mut templates = vec![];
        
        for i in 0..10000 {
            let template = IndexTemplate {
                title: format!("Test {}", i),
            };
            templates.push(template);
        }
        
        // 验证可以创建大量实例而不会内存溢出
        assert_eq!(templates.len(), 10000);
    }

    #[actix_web::test]
    async fn test_sequential_requests_performance() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        let start = Instant::now();
        
        // 顺序发送 100 个请求
        for _ in 0..100 {
            let req = test::TestRequest::get()
                .uri("/clawmesh/")
                .to_request();
            
            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success());
        }
        
        let duration = start.elapsed();
        
        // 100 个顺序请求应该在 5 秒内完成
        assert!(duration.as_secs() < 5, "Sequential requests took: {:?}", duration);
    }

    #[actix_web::test]
    async fn test_i18n_page_load_performance() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let languages = vec!["en", "zh-CN", "ja", "ko"];
        
        for lang in languages {
            let start = Instant::now();
            
            let uri = format!("/clawmesh/i18n/{}/", lang);
            let req = test::TestRequest::get()
                .uri(&uri)
                .to_request();
            
            let resp = test::call_service(&app, req).await;
            
            let duration = start.elapsed();
            
            assert!(resp.status().is_success());
            // 国际化页面加载时间应该小于 250ms
            assert!(duration.as_millis() < 250, "i18n page load time for {}: {:?}", lang, duration);
        }
    }

    #[actix_web::test]
    async fn test_error_page_performance() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        let start = Instant::now();
        
        let req = test::TestRequest::get()
            .uri("/clawmesh/nonexistent")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        
        let duration = start.elapsed();
        
        // 错误页面也应该快速响应
        assert!(duration.as_millis() < 100, "Error page load time: {:?}", duration);
    }

    #[actix_web::test]
    async fn test_large_title_performance() {
        use clawmesh_ui::templates::*;
        use askama::Template;
        
        // 测试大标题的渲染性能
        let large_title = "A".repeat(10000);
        
        let start = Instant::now();
        
        let template = IndexTemplate {
            title: large_title,
        };
        
        let _html = template.render().expect("Failed to render");
        
        let duration = start.elapsed();
        
        // 即使标题很大，渲染也应该很快
        assert!(duration.as_millis() < 50, "Large title render time: {:?}", duration);
    }

    #[actix_web::test]
    async fn test_cache_effectiveness() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        // 第一次请求
        let start1 = Instant::now();
        let req1 = test::TestRequest::get()
            .uri("/clawmesh/")
            .to_request();
        let _resp1 = test::call_service(&app, req1).await;
        let duration1 = start1.elapsed();

        // 第二次请求（应该更快，如果有缓存）
        let start2 = Instant::now();
        let req2 = test::TestRequest::get()
            .uri("/clawmesh/")
            .to_request();
        let _resp2 = test::call_service(&app, req2).await;
        let duration2 = start2.elapsed();

        // 第二次请求应该不会明显慢于第一次
        assert!(duration2.as_millis() <= duration1.as_millis() + 50);
    }
}
