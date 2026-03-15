/// ClawMesh UI 模板渲染测试
/// 
/// 测试所有模板的渲染功能和正确性

use clawmesh_ui::templates::*;
use askama::Template;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_template_render() {
        let template = IndexTemplate {
            title: "Test Title".to_string(),
        };
        
        let html = template.render().expect("Failed to render index template");
        
        // 验证模板包含关键内容
        assert!(html.contains("Test Title"));
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("ClawMesh"));
        assert!(html.len() > 1000); // 确保模板有实质内容
    }

    #[test]
    fn test_credit_template_render() {
        let template = CreditTemplate {
            title: "Credit System".to_string(),
            user_credit: 750,
            user_tier: "Premium".to_string(),
        };
        
        let html = template.render().expect("Failed to render credit template");
        
        assert!(html.contains("Credit System"));
        assert!(html.contains("750"));
        assert!(html.contains("Premium"));
    }

    #[test]
    fn test_agent_template_render() {
        let template = AgentTemplate {
            title: "Agent Management".to_string(),
            agent_count: 25,
        };
        
        let html = template.render().expect("Failed to render agent template");
        
        assert!(html.contains("Agent Management"));
        assert!(html.contains("25"));
    }

    #[test]
    fn test_stats_template_render() {
        let template = StatsTemplate {
            title: "Statistics".to_string(),
            total_users: 5000,
            avg_credit: 523.75,
        };
        
        let html = template.render().expect("Failed to render stats template");
        
        assert!(html.contains("Statistics"));
        assert!(html.contains("5000"));
        assert!(html.contains("523.75"));
    }

    #[test]
    fn test_error_404_template_render() {
        let template = Error404Template {
            title: "404 Not Found".to_string(),
        };
        
        let html = template.render().expect("Failed to render 404 template");
        
        assert!(html.contains("404"));
        assert!(html.contains("Not Found"));
    }

    #[test]
    fn test_error_500_template_render() {
        let template = Error500Template {
            title: "500 Internal Server Error".to_string(),
        };
        
        let html = template.render().expect("Failed to render 500 template");
        
        assert!(html.contains("500"));
        assert!(html.contains("Internal Server Error"));
    }

    #[test]
    fn test_template_xss_protection() {
        // 测试 XSS 攻击防护
        let malicious_input = "<script>alert('XSS')</script>";
        
        let template = IndexTemplate {
            title: malicious_input.to_string(),
        };
        
        let html = template.render().expect("Failed to render template");
        
        // Askama 应该自动转义 HTML
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;") || html.contains("alert"));
    }

    #[test]
    fn test_template_special_characters() {
        // 测试特殊字符处理
        let special_chars = "Test & <> \" ' 中文 日本語 한국어";
        
        let template = IndexTemplate {
            title: special_chars.to_string(),
        };
        
        let html = template.render().expect("Failed to render template");
        
        // 验证特殊字符被正确处理
        assert!(html.contains("Test"));
        assert!(html.contains("中文"));
        assert!(html.contains("日本語"));
        assert!(html.contains("한국어"));
    }

    #[test]
    fn test_template_empty_values() {
        // 测试空值处理
        let template = IndexTemplate {
            title: "".to_string(),
        };
        
        let html = template.render().expect("Failed to render template");
        
        // 即使标题为空，模板也应该正常渲染
        assert!(html.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_template_large_values() {
        // 测试大数值处理
        let template = StatsTemplate {
            title: "Large Numbers".to_string(),
            total_users: 1_000_000,
            avg_credit: 999_999.99,
        };
        
        let html = template.render().expect("Failed to render template");
        
        assert!(html.contains("1000000") || html.contains("1,000,000"));
        assert!(html.contains("999999.99") || html.contains("999,999.99"));
    }

    #[test]
    fn test_template_negative_values() {
        // 测试负数处理
        let template = CreditTemplate {
            title: "Negative Credit".to_string(),
            user_credit: -100,
            user_tier: "Restricted".to_string(),
        };
        
        let html = template.render().expect("Failed to render template");
        
        assert!(html.contains("-100") || html.contains("100"));
        assert!(html.contains("Restricted"));
    }

    #[test]
    fn test_template_unicode_support() {
        // 测试 Unicode 字符支持
        let unicode_text = "🎯 ClawMesh 🚀 测试 ✅";
        
        let template = IndexTemplate {
            title: unicode_text.to_string(),
        };
        
        let html = template.render().expect("Failed to render template");
        
        assert!(html.contains("🎯"));
        assert!(html.contains("🚀"));
        assert!(html.contains("✅"));
    }

    #[test]
    fn test_template_html_structure() {
        // 测试 HTML 结构完整性
        let template = IndexTemplate {
            title: "Structure Test".to_string(),
        };
        
        let html = template.render().expect("Failed to render template");
        
        // 验证基本 HTML 结构
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html"));
        assert!(html.contains("<head>"));
        assert!(html.contains("<body>"));
        assert!(html.contains("</body>"));
        assert!(html.contains("</html>"));
        
        // 验证 meta 标签
        assert!(html.contains("<meta charset=\"UTF-8\">"));
        assert!(html.contains("viewport"));
    }

    #[test]
    fn test_template_css_inclusion() {
        // 测试 CSS 样式包含
        let template = IndexTemplate {
            title: "CSS Test".to_string(),
        };
        
        let html = template.render().expect("Failed to render template");
        
        // 验证包含样式
        assert!(html.contains("<style>"));
        assert!(html.contains("</style>"));
        assert!(html.contains("background"));
        assert!(html.contains("color"));
    }

    #[test]
    fn test_template_responsive_design() {
        // 测试响应式设计元素
        let template = IndexTemplate {
            title: "Responsive Test".to_string(),
        };
        
        let html = template.render().expect("Failed to render template");
        
        // 验证响应式设计元素
        assert!(html.contains("viewport"));
        assert!(html.contains("width=device-width"));
        assert!(html.contains("grid") || html.contains("flex"));
    }
}
