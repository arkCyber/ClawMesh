/// HTML 模板定义
/// 
/// 使用 Askama 模板引擎

use askama::Template;

/// 首页模板
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub title: String,
}

/// 信用系统页面模板
#[derive(Template)]
#[template(path = "credit.html")]
pub struct CreditTemplate {
    pub title: String,
    pub user_credit: i32,
    pub user_tier: String,
}

/// 智能体管理页面模板
#[derive(Template)]
#[template(path = "agent.html")]
pub struct AgentTemplate {
    pub title: String,
    pub agent_count: i64,
}

/// 统计页面模板
#[derive(Template)]
#[template(path = "stats.html")]
pub struct StatsTemplate {
    pub title: String,
    pub total_users: i64,
    pub avg_credit: f64,
}
