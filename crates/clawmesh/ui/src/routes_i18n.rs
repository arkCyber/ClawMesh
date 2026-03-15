/// 支持多语言的 UI 路由处理器

use actix_web::{HttpRequest, HttpResponse, Result};
use askama::Template;
use crate::i18n::Language;
use crate::templates_i18n::*;

/// 从请求中获取语言偏好
fn get_language_from_request(req: &HttpRequest) -> Language {
    // 1. 尝试从查询参数获取
    if let Some(query) = req.uri().query() {
        for param in query.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                if key == "lang" {
                    return Language::from_str(value);
                }
            }
        }
    }
    
    // 2. 尝试从 Cookie 获取
    if let Some(cookie) = req.cookie("lang") {
        return Language::from_str(cookie.value());
    }
    
    // 3. 尝试从 Accept-Language 头获取
    if let Some(accept_lang) = req.headers().get("Accept-Language") {
        if let Ok(lang_str) = accept_lang.to_str() {
            // 解析 Accept-Language: zh-CN,zh;q=0.9,en;q=0.8
            let first_lang = lang_str.split(',').next().unwrap_or("zh-CN");
            let lang = first_lang.split(';').next().unwrap_or("zh-CN");
            return Language::from_str(lang);
        }
    }
    
    // 4. 默认中文
    Language::ZhCN
}

/// 首页（多语言）
pub async fn index_i18n(req: HttpRequest) -> Result<HttpResponse> {
    let language = get_language_from_request(&req);
    let template = IndexI18nTemplate::new(language);
    
    let html = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;
    
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

/// 信用系统页面（多语言）
pub async fn credit_page_i18n(req: HttpRequest) -> Result<HttpResponse> {
    let language = get_language_from_request(&req);
    let template = CreditI18nTemplate::new(language, 500, "Regular".to_string());
    
    let html = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;
    
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

/// 智能体管理页面（多语言）
pub async fn agent_page_i18n(req: HttpRequest) -> Result<HttpResponse> {
    let language = get_language_from_request(&req);
    let template = AgentI18nTemplate::new(language, 10);
    
    let html = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;
    
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

/// 统计页面（多语言）
pub async fn stats_page_i18n(req: HttpRequest) -> Result<HttpResponse> {
    let language = get_language_from_request(&req);
    let template = StatsI18nTemplate::new(language, 1000, 450.5);
    
    let html = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;
    
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

/// 配置多语言路由
pub fn config_i18n(cfg: &mut actix_web::web::ServiceConfig) {
    use actix_web::web;
    
    cfg.service(
        web::scope("/clawmesh/i18n")
            .route("/", web::get().to(index_i18n))
            .route("/credit", web::get().to(credit_page_i18n))
            .route("/agent", web::get().to(agent_page_i18n))
            .route("/stats", web::get().to(stats_page_i18n))
    );
}
