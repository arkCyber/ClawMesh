/// ClawMesh Web UI
/// 
/// 使用 Rust + Askama 模板引擎实现的 Web 界面

use actix_web::{web, HttpResponse, Result};
use askama::Template;

pub mod templates;
pub mod routes;
pub mod routes_with_db;
pub mod error_handlers;
pub mod i18n;
pub mod templates_i18n;
pub mod routes_i18n;

// 导出多语言配置函数
pub use routes_i18n::config_i18n;

/// 配置 UI 路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/clawmesh")
            .route("/", web::get().to(routes::index))
            .route("/credit", web::get().to(routes::credit_page))
            .route("/agent", web::get().to(routes::agent_page))
            .route("/stats", web::get().to(routes::stats_page))
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_module_exists() {
        assert!(true);
    }
}
