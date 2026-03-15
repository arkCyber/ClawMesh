/// 错误处理器

use actix_web::{HttpResponse, Result};

/// 404 错误页面
pub async fn not_found() -> Result<HttpResponse> {
    let html = include_str!("../templates/error_404.html");
    Ok(HttpResponse::NotFound()
        .content_type("text/html")
        .body(html))
}

/// 500 错误页面
pub async fn internal_error() -> Result<HttpResponse> {
    let html = include_str!("../templates/error_500.html");
    Ok(HttpResponse::InternalServerError()
        .content_type("text/html")
        .body(html))
}
