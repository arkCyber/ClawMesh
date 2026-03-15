/// UI 路由处理器

use actix_web::{HttpResponse, Result};
use askama::Template;
use crate::templates::*;

/// 首页
pub async fn index() -> Result<HttpResponse> {
    let template = IndexTemplate {
        title: "ClawMesh - 智能社区管理系统".to_string(),
    };
    
    let html = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;
    
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

/// 信用系统页面
pub async fn credit_page() -> Result<HttpResponse> {
    let template = CreditTemplate {
        title: "信用系统".to_string(),
        user_credit: 500,
        user_tier: "Regular".to_string(),
    };
    
    let html = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;
    
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

/// 智能体管理页面
pub async fn agent_page() -> Result<HttpResponse> {
    let template = AgentTemplate {
        title: "智能体管理".to_string(),
        agent_count: 10,
    };
    
    let html = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;
    
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

/// 统计页面
pub async fn stats_page() -> Result<HttpResponse> {
    let template = StatsTemplate {
        title: "系统统计".to_string(),
        total_users: 1000,
        avg_credit: 450.5,
    };
    
    let html = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;
    
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}
