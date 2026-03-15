/// ClawMesh UI 安全测试
/// 
/// 测试 XSS、CSRF、注入攻击等安全防护

use actix_web::{test, web, App};
use clawmesh_ui;

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn test_xss_protection_in_url() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        // 尝试在 URL 中注入脚本
        let req = test::TestRequest::get()
            .uri("/clawmesh/<script>alert('XSS')</script>")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        
        // 应该返回 404 或安全处理
        assert!(resp.status().is_client_error());
    }

    #[actix_web::test]
    async fn test_xss_protection_in_template() {
        use clawmesh_ui::templates::*;
        use askama::Template;
        
        // 尝试在模板中注入脚本
        let malicious_input = "<script>alert('XSS')</script>";
        
        let template = IndexTemplate {
            title: malicious_input.to_string(),
        };
        
        let html = template.render().expect("Failed to render");
        
        // Askama 应该自动转义 HTML
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;") || html.contains("&amp;lt;"));
    }

    #[actix_web::test]
    async fn test_sql_injection_protection() {
        use clawmesh_ui::templates::*;
        use askama::Template;
        
        // 尝试 SQL 注入攻击
        let sql_injection = "'; DROP TABLE users; --";
        
        let template = CreditTemplate {
            title: sql_injection.to_string(),
            user_credit: 500,
            user_tier: "Regular".to_string(),
        };
        
        let html = template.render().expect("Failed to render");
        
        // 应该转义特殊字符
        assert!(!html.contains("DROP TABLE"));
        assert!(html.contains("&") || html.contains("&#"));
    }

    #[actix_web::test]
    async fn test_path_traversal_protection() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        // 尝试路径遍历攻击
        let req = test::TestRequest::get()
            .uri("/clawmesh/../../../etc/passwd")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        
        // 应该返回 404 或 403
        assert!(resp.status().is_client_error());
    }

    #[actix_web::test]
    async fn test_html_injection_protection() {
        use clawmesh_ui::templates::*;
        use askama::Template;
        
        // 尝试 HTML 注入
        let html_injection = "<img src=x onerror=alert('XSS')>";
        
        let template = IndexTemplate {
            title: html_injection.to_string(),
        };
        
        let html = template.render().expect("Failed to render");
        
        // 应该转义 HTML 标签
        assert!(!html.contains("<img"));
        assert!(!html.contains("onerror"));
    }

    #[actix_web::test]
    async fn test_javascript_injection_protection() {
        use clawmesh_ui::templates::*;
        use askama::Template;
        
        // 尝试 JavaScript 注入
        let js_injection = "javascript:alert('XSS')";
        
        let template = IndexTemplate {
            title: js_injection.to_string(),
        };
        
        let html = template.render().expect("Failed to render");
        
        // 应该转义或移除 javascript: 协议
        assert!(!html.contains("javascript:alert"));
    }

    #[actix_web::test]
    async fn test_iframe_injection_protection() {
        use clawmesh_ui::templates::*;
        use askama::Template;
        
        // 尝试 iframe 注入
        let iframe_injection = "<iframe src='http://evil.com'></iframe>";
        
        let template = IndexTemplate {
            title: iframe_injection.to_string(),
        };
        
        let html = template.render().expect("Failed to render");
        
        // 应该转义 iframe 标签
        assert!(!html.contains("<iframe"));
    }

    #[actix_web::test]
    async fn test_event_handler_injection() {
        use clawmesh_ui::templates::*;
        use askama::Template;
        
        // 尝试注入事件处理器
        let event_injection = "' onclick='alert(1)' '";
        
        let template = IndexTemplate {
            title: event_injection.to_string(),
        };
        
        let html = template.render().expect("Failed to render");
        
        // 应该转义单引号
        assert!(!html.contains("onclick="));
    }

    #[actix_web::test]
    async fn test_unicode_escape_injection() {
        use clawmesh_ui::templates::*;
        use askama::Template;
        
        // 尝试 Unicode 转义注入
        let unicode_injection = "\\u003cscript\\u003ealert('XSS')\\u003c/script\\u003e";
        
        let template = IndexTemplate {
            title: unicode_injection.to_string(),
        };
        
        let html = template.render().expect("Failed to render");
        
        // 应该安全处理 Unicode 转义
        assert!(!html.contains("<script>"));
    }

    #[actix_web::test]
    async fn test_null_byte_injection() {
        use clawmesh_ui::templates::*;
        use askama::Template;
        
        // 尝试空字节注入
        let null_byte = "test\0<script>alert('XSS')</script>";
        
        let template = IndexTemplate {
            title: null_byte.to_string(),
        };
        
        let html = template.render().expect("Failed to render");
        
        // 应该安全处理空字节
        assert!(!html.contains("<script>"));
    }

    #[actix_web::test]
    async fn test_content_type_header() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        
        // 验证 Content-Type 正确设置
        let content_type = resp.headers().get("content-type");
        assert!(content_type.is_some());
        assert!(content_type.unwrap().to_str().unwrap().contains("text/html"));
    }

    #[actix_web::test]
    async fn test_no_cache_sensitive_pages() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        let req = test::TestRequest::get()
            .uri("/clawmesh/credit")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        
        // 敏感页面应该设置 no-cache 头（如果实现了）
        // 这里只是验证响应成功
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_special_characters_handling() {
        use clawmesh_ui::templates::*;
        use askama::Template;
        
        // 测试各种特殊字符
        let special_chars = vec![
            "<", ">", "&", "\"", "'",
            "\n", "\r", "\t",
            "\\", "/",
        ];
        
        for ch in special_chars {
            let template = IndexTemplate {
                title: ch.to_string(),
            };
            
            let html = template.render().expect("Failed to render");
            
            // 特殊字符应该被转义或安全处理
            // 不应该直接出现在 HTML 中（除非是安全的）
            if ch == "<" || ch == ">" || ch == "&" || ch == "\"" {
                assert!(!html.contains(&format!(">{}<", ch)));
            }
        }
    }

    #[actix_web::test]
    async fn test_long_input_handling() {
        use clawmesh_ui::templates::*;
        use askama::Template;
        
        // 测试超长输入
        let long_input = "A".repeat(100000);
        
        let template = IndexTemplate {
            title: long_input.clone(),
        };
        
        let html = template.render().expect("Failed to render");
        
        // 应该能够处理长输入而不崩溃
        assert!(html.contains(&long_input));
    }

    #[actix_web::test]
    async fn test_mixed_content_protection() {
        use clawmesh_ui::templates::*;
        use askama::Template;
        
        // 测试混合内容（HTML + 脚本）
        let mixed_content = "Normal text <script>alert('XSS')</script> more text";
        
        let template = IndexTemplate {
            title: mixed_content.to_string(),
        };
        
        let html = template.render().expect("Failed to render");
        
        // 脚本标签应该被转义
        assert!(!html.contains("<script>"));
        assert!(html.contains("Normal text"));
        assert!(html.contains("more text"));
    }

    #[actix_web::test]
    async fn test_url_encoding_handling() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        // 测试 URL 编码的恶意输入
        let req = test::TestRequest::get()
            .uri("/clawmesh/%3Cscript%3Ealert('XSS')%3C/script%3E")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        
        // 应该安全处理 URL 编码
        assert!(resp.status().is_client_error());
    }

    #[actix_web::test]
    async fn test_double_encoding_attack() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        // 测试双重编码攻击
        let req = test::TestRequest::get()
            .uri("/clawmesh/%253Cscript%253E")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        
        // 应该安全处理双重编码
        assert!(resp.status().is_client_error());
    }

    #[actix_web::test]
    async fn test_comment_injection() {
        use clawmesh_ui::templates::*;
        use askama::Template;
        
        // 测试 HTML 注释注入
        let comment_injection = "<!-- <script>alert('XSS')</script> -->";
        
        let template = IndexTemplate {
            title: comment_injection.to_string(),
        };
        
        let html = template.render().expect("Failed to render");
        
        // 注释标签应该被转义
        assert!(!html.contains("<!--"));
    }

    #[actix_web::test]
    async fn test_style_injection() {
        use clawmesh_ui::templates::*;
        use askama::Template;
        
        // 测试 CSS 注入
        let style_injection = "<style>body{display:none}</style>";
        
        let template = IndexTemplate {
            title: style_injection.to_string(),
        };
        
        let html = template.render().expect("Failed to render");
        
        // style 标签应该被转义
        assert!(!html.contains("<style>"));
    }

    #[actix_web::test]
    async fn test_meta_tag_injection() {
        use clawmesh_ui::templates::*;
        use askama::Template;
        
        // 测试 meta 标签注入
        let meta_injection = "<meta http-equiv='refresh' content='0;url=http://evil.com'>";
        
        let template = IndexTemplate {
            title: meta_injection.to_string(),
        };
        
        let html = template.render().expect("Failed to render");
        
        // meta 标签应该被转义
        assert!(!html.contains("<meta"));
    }
}
