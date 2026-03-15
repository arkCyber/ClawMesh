/// UI 路由处理器（带数据库集成）
/// 
/// 这是集成真实数据库的版本

use actix_web::{web, HttpResponse, Result};
use askama::Template;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::AsyncPgConnection;
use lemmy_db_schema::source::person::Person;
use lemmy_db_schema_file::PersonId;
use crate::templates::*;

/// 数据库连接池类型
pub type DbPool = Pool<AsyncPgConnection>;

/// 首页（无需数据库）
pub async fn index() -> Result<HttpResponse> {
    let template = IndexTemplate {
        title: "ClawMesh - 智能社区管理系统".to_string(),
    };
    
    let html = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;
    
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

/// 信用系统页面（集成真实数据）
pub async fn credit_page_with_db(
    pool: web::Data<DbPool>,
    person_id: web::Path<i32>,
) -> Result<HttpResponse> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use lemmy_db_schema_file::schema::person;
    
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    // 从数据库获取用户信息
    let person: Person = person::table
        .find(PersonId(person_id.into_inner()))
        .first(&mut conn)
        .await
        .map_err(|e| {
            actix_web::error::ErrorNotFound(format!("Person not found: {}", e))
        })?;
    
    let template = CreditTemplate {
        title: "信用系统".to_string(),
        user_credit: person.credit_score,
        user_tier: person.reputation_tier,
    };
    
    let html = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;
    
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

/// 智能体管理页面（集成真实数据）
pub async fn agent_page_with_db(
    pool: web::Data<DbPool>,
) -> Result<HttpResponse> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use lemmy_db_schema_file::schema::person;
    
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    // 统计智能体数量
    let agent_count: i64 = person::table
        .filter(person::user_type.eq("agent"))
        .count()
        .get_result(&mut conn)
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
        })?;
    
    let template = AgentTemplate {
        title: "智能体管理".to_string(),
        agent_count,
    };
    
    let html = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;
    
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

/// 统计页面（集成真实数据）
pub async fn stats_page_with_db(
    pool: web::Data<DbPool>,
) -> Result<HttpResponse> {
    use diesel::prelude::*;
    use diesel::dsl::*;
    use diesel_async::RunQueryDsl;
    use lemmy_db_schema_file::schema::person;
    
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    // 获取统计数据
    let total_users: i64 = person::table
        .count()
        .get_result(&mut conn)
        .await
        .unwrap_or(0);
    
    // 使用简单的方式计算平均值：先获取总和和计数
    let credit_sum: i64 = person::table
        .select(sum(person::credit_score))
        .first::<Option<i64>>(&mut conn)
        .await
        .unwrap_or(Some(0))
        .unwrap_or(0);
    
    let avg_credit: f64 = if total_users > 0 {
        credit_sum as f64 / total_users as f64
    } else {
        0.0
    };
    
    let template = StatsTemplate {
        title: "系统统计".to_string(),
        total_users,
        avg_credit,
    };
    
    let html = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;
    
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

/// 配置带数据库的路由
pub fn config_with_db(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/clawmesh")
            .route("/", web::get().to(index))
            .route("/credit/{person_id}", web::get().to(credit_page_with_db))
            .route("/agent", web::get().to(agent_page_with_db))
            .route("/stats", web::get().to(stats_page_with_db))
    );
}
