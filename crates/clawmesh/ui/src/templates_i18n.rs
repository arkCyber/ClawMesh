/// 支持多语言的模板定义

use askama::Template;
use crate::i18n::{Language, Translator};

/// 首页模板（多语言）
#[derive(Template)]
#[template(path = "index_i18n.html")]
pub struct IndexI18nTemplate {
    pub translator: Translator,
}

impl IndexI18nTemplate {
    pub fn new(language: Language) -> Self {
        IndexI18nTemplate {
            translator: Translator::new(language),
        }
    }
}

/// 信用系统页面模板（多语言）
#[derive(Template)]
#[template(path = "credit_i18n.html")]
pub struct CreditI18nTemplate {
    pub translator: Translator,
    pub user_credit: i32,
    pub user_tier: String,
}

impl CreditI18nTemplate {
    pub fn new(language: Language, user_credit: i32, user_tier: String) -> Self {
        CreditI18nTemplate {
            translator: Translator::new(language),
            user_credit,
            user_tier,
        }
    }
}

/// 智能体管理页面模板（多语言）
#[derive(Template)]
#[template(path = "agent_i18n.html")]
pub struct AgentI18nTemplate {
    pub translator: Translator,
    pub agent_count: i64,
}

impl AgentI18nTemplate {
    pub fn new(language: Language, agent_count: i64) -> Self {
        AgentI18nTemplate {
            translator: Translator::new(language),
            agent_count,
        }
    }
}

/// 统计页面模板（多语言）
#[derive(Template)]
#[template(path = "stats_i18n.html")]
pub struct StatsI18nTemplate {
    pub translator: Translator,
    pub total_users: i64,
    pub avg_credit: f64,
}

impl StatsI18nTemplate {
    pub fn new(language: Language, total_users: i64, avg_credit: f64) -> Self {
        StatsI18nTemplate {
            translator: Translator::new(language),
            total_users,
            avg_credit,
        }
    }
}
