/// ClawMesh UI 国际化 (i18n) 测试
/// 
/// 测试所有语言的翻译和国际化功能

use actix_web::{test, web, App};
use clawmesh_ui;

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn test_i18n_english() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/en/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let html = String::from_utf8(body.to_vec()).unwrap();
        
        // 验证英文内容
        assert!(html.contains("ClawMesh") || html.contains("Welcome"));
    }

    #[actix_web::test]
    async fn test_i18n_chinese_simplified() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/zh-CN/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let html = String::from_utf8(body.to_vec()).unwrap();
        
        // 验证中文简体内容
        assert!(html.contains("欢迎") || html.contains("系统"));
    }

    #[actix_web::test]
    async fn test_i18n_chinese_traditional() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/zh-TW/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_i18n_japanese() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/ja/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let html = String::from_utf8(body.to_vec()).unwrap();
        
        // 验证日文内容
        assert!(html.contains("システム") || html.contains("ようこそ"));
    }

    #[actix_web::test]
    async fn test_i18n_korean() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/ko/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let html = String::from_utf8(body.to_vec()).unwrap();
        
        // 验证韩文内容
        assert!(html.contains("시스템") || html.contains("환영"));
    }

    #[actix_web::test]
    async fn test_i18n_french() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/fr/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_i18n_german() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/de/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_i18n_spanish() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/es/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_i18n_russian() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/ru/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_i18n_arabic() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/ar/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_i18n_portuguese() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/pt/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_i18n_italian() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/it/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_i18n_dutch() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/nl/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_i18n_polish() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/pl/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_i18n_turkish() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/tr/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_i18n_hindi() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/hi/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_i18n_invalid_language() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/invalid/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        
        // 无效语言应该返回 404 或重定向到默认语言
        assert!(resp.status().is_client_error() || resp.status().is_redirection());
    }

    #[actix_web::test]
    async fn test_i18n_credit_page_all_languages() {
        let languages = vec!["en", "zh-CN", "ja", "ko", "fr", "de", "es", "ru"];
        
        for lang in languages {
            let app = test::init_service(
                App::new().configure(clawmesh_ui::config_i18n)
            ).await;

            let uri = format!("/clawmesh/i18n/{}/credit", lang);
            let req = test::TestRequest::get()
                .uri(&uri)
                .to_request();
            
            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success(), "Failed for language: {}", lang);
        }
    }

    #[actix_web::test]
    async fn test_i18n_agent_page_all_languages() {
        let languages = vec!["en", "zh-CN", "ja", "ko", "fr", "de", "es", "ru"];
        
        for lang in languages {
            let app = test::init_service(
                App::new().configure(clawmesh_ui::config_i18n)
            ).await;

            let uri = format!("/clawmesh/i18n/{}/agent", lang);
            let req = test::TestRequest::get()
                .uri(&uri)
                .to_request();
            
            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success(), "Failed for language: {}", lang);
        }
    }

    #[actix_web::test]
    async fn test_i18n_stats_page_all_languages() {
        let languages = vec!["en", "zh-CN", "ja", "ko", "fr", "de", "es", "ru"];
        
        for lang in languages {
            let app = test::init_service(
                App::new().configure(clawmesh_ui::config_i18n)
            ).await;

            let uri = format!("/clawmesh/i18n/{}/stats", lang);
            let req = test::TestRequest::get()
                .uri(&uri)
                .to_request();
            
            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success(), "Failed for language: {}", lang);
        }
    }

    #[actix_web::test]
    async fn test_i18n_content_type() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/en/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        
        // 验证 Content-Type 是 text/html
        let content_type = resp.headers().get("content-type");
        assert!(content_type.is_some());
        assert!(content_type.unwrap().to_str().unwrap().contains("text/html"));
    }

    #[actix_web::test]
    async fn test_i18n_charset_utf8() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config_i18n)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/i18n/zh-CN/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        let html = String::from_utf8(body.to_vec()).unwrap();
        
        // 验证 UTF-8 编码声明
        assert!(html.contains("charset=\"UTF-8\"") || html.contains("charset=UTF-8"));
    }
}
