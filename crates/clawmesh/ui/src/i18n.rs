/// ClawMesh UI 国际化（i18n）系统
/// 
/// 支持多语言界面

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 支持的语言（16种）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    /// 中文（简体）
    ZhCN,
    /// 英文
    En,
    /// 日语
    Ja,
    /// 韩语
    Ko,
    /// 法语
    Fr,
    /// 德语
    De,
    /// 西班牙语
    Es,
    /// 葡萄牙语
    Pt,
    /// 俄语
    Ru,
    /// 阿拉伯语
    Ar,
    /// 印地语
    Hi,
    /// 意大利语
    It,
    /// 荷兰语
    Nl,
    /// 土耳其语
    Tr,
    /// 波兰语
    Pl,
    /// 越南语
    Vi,
}

impl Language {
    /// 从字符串解析语言
    pub fn from_str(s: &str) -> Self {
        match s {
            "zh-CN" | "zh" | "中文" => Language::ZhCN,
            "en" | "en-US" | "English" => Language::En,
            "ja" | "ja-JP" | "日本語" => Language::Ja,
            "ko" | "ko-KR" | "한국어" => Language::Ko,
            "fr" | "fr-FR" | "Français" => Language::Fr,
            "de" | "de-DE" | "Deutsch" => Language::De,
            "es" | "es-ES" | "Español" => Language::Es,
            "pt" | "pt-BR" | "pt-PT" | "Português" => Language::Pt,
            "ru" | "ru-RU" | "Русский" => Language::Ru,
            "ar" | "ar-SA" | "العربية" => Language::Ar,
            "hi" | "hi-IN" | "हिन्दी" => Language::Hi,
            "it" | "it-IT" | "Italiano" => Language::It,
            "nl" | "nl-NL" | "Nederlands" => Language::Nl,
            "tr" | "tr-TR" | "Türkçe" => Language::Tr,
            "pl" | "pl-PL" | "Polski" => Language::Pl,
            "vi" | "vi-VN" | "Tiếng Việt" => Language::Vi,
            _ => Language::ZhCN, // 默认中文
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::ZhCN => "zh-CN",
            Language::En => "en",
            Language::Ja => "ja",
            Language::Ko => "ko",
            Language::Fr => "fr",
            Language::De => "de",
            Language::Es => "es",
            Language::Pt => "pt",
            Language::Ru => "ru",
            Language::Ar => "ar",
            Language::Hi => "hi",
            Language::It => "it",
            Language::Nl => "nl",
            Language::Tr => "tr",
            Language::Pl => "pl",
            Language::Vi => "vi",
        }
    }

    /// 获取语言名称
    pub fn name(&self) -> &'static str {
        match self {
            Language::ZhCN => "中文",
            Language::En => "English",
            Language::Ja => "日本語",
            Language::Ko => "한국어",
            Language::Fr => "Français",
            Language::De => "Deutsch",
            Language::Es => "Español",
            Language::Pt => "Português",
            Language::Ru => "Русский",
            Language::Ar => "العربية",
            Language::Hi => "हिन्दी",
            Language::It => "Italiano",
            Language::Nl => "Nederlands",
            Language::Tr => "Türkçe",
            Language::Pl => "Polski",
            Language::Vi => "Tiếng Việt",
        }
    }
}

/// 翻译键
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TranslationKey(String);

impl TranslationKey {
    pub fn new(key: &str) -> Self {
        TranslationKey(key.to_string())
    }
}

impl From<&str> for TranslationKey {
    fn from(s: &str) -> Self {
        TranslationKey::new(s)
    }
}

/// 翻译器
#[derive(Debug, Clone)]
pub struct Translator {
    language: Language,
    translations: HashMap<String, String>,
}

impl Translator {
    /// 创建新的翻译器
    pub fn new(language: Language) -> Self {
        let translations = Self::load_translations(language);
        Translator {
            language,
            translations,
        }
    }

    /// 加载翻译
    fn load_translations(language: Language) -> HashMap<String, String> {
        match language {
            Language::ZhCN => Self::zh_cn_translations(),
            Language::En => Self::en_translations(),
            Language::Ja => Self::ja_translations(),
            Language::Ko => Self::ko_translations(),
            Language::Fr => Self::fr_translations(),
            Language::De => Self::de_translations(),
            Language::Es => Self::es_translations(),
            Language::Pt => Self::pt_translations(),
            Language::Ru => Self::ru_translations(),
            Language::Ar => Self::ar_translations(),
            Language::Hi => Self::hi_translations(),
            Language::It => Self::it_translations(),
            Language::Nl => Self::nl_translations(),
            Language::Tr => Self::tr_translations(),
            Language::Pl => Self::pl_translations(),
            Language::Vi => Self::vi_translations(),
        }
    }

    /// 中文翻译
    fn zh_cn_translations() -> HashMap<String, String> {
        let mut map = HashMap::new();
        
        // 通用
        map.insert("app.name".to_string(), "ClawMesh".to_string());
        map.insert("app.subtitle".to_string(), "智能社区管理系统".to_string());
        map.insert("app.description".to_string(), "基于 Rust 构建的智能社区管理系统，提供信用评分、智能体管理和数据分析功能".to_string());
        map.insert("app.version".to_string(), "v1.0.0".to_string());
        map.insert("app.powered_by".to_string(), "Powered by Rust 🦀".to_string());
        
        // 导航
        map.insert("nav.home".to_string(), "首页".to_string());
        map.insert("nav.back".to_string(), "返回首页".to_string());
        map.insert("nav.credit".to_string(), "信用系统".to_string());
        map.insert("nav.agent".to_string(), "智能体管理".to_string());
        map.insert("nav.stats".to_string(), "数据统计".to_string());
        
        // 首页
        map.insert("home.welcome".to_string(), "欢迎使用 ClawMesh".to_string());
        map.insert("home.credit.title".to_string(), "信用系统".to_string());
        map.insert("home.credit.desc".to_string(), "查看和管理用户信用分数，追踪声誉等级变化，实时监控社区质量".to_string());
        map.insert("home.agent.title".to_string(), "智能体管理".to_string());
        map.insert("home.agent.desc".to_string(), "管理和监控智能体，查看心跳状态，配置自动化任务".to_string());
        map.insert("home.stats.title".to_string(), "数据统计".to_string());
        map.insert("home.stats.desc".to_string(), "查看全局统计数据，分析用户行为，优化社区运营".to_string());
        
        // 信用系统
        map.insert("credit.title".to_string(), "信用系统".to_string());
        map.insert("credit.score".to_string(), "信用分数".to_string());
        map.insert("credit.tier".to_string(), "声誉等级".to_string());
        map.insert("credit.next_tier".to_string(), "下一等级".to_string());
        map.insert("credit.needed".to_string(), "还需信用".to_string());
        map.insert("credit.rank".to_string(), "当前排名".to_string());
        map.insert("credit.api_endpoints".to_string(), "API 端点".to_string());
        
        // 智能体
        map.insert("agent.title".to_string(), "智能体管理".to_string());
        map.insert("agent.total".to_string(), "总智能体数".to_string());
        map.insert("agent.active".to_string(), "活跃智能体".to_string());
        map.insert("agent.inactive".to_string(), "不活跃智能体".to_string());
        map.insert("agent.list".to_string(), "智能体列表".to_string());
        map.insert("agent.status.active".to_string(), "活跃".to_string());
        map.insert("agent.status.inactive".to_string(), "不活跃".to_string());
        map.insert("agent.heartbeat_interval".to_string(), "心跳间隔".to_string());
        map.insert("agent.last_heartbeat".to_string(), "最后心跳".to_string());
        
        // 统计
        map.insert("stats.title".to_string(), "系统统计".to_string());
        map.insert("stats.total_users".to_string(), "总用户数".to_string());
        map.insert("stats.avg_credit".to_string(), "平均信用分".to_string());
        map.insert("stats.active_agents".to_string(), "活跃智能体".to_string());
        map.insert("stats.growth".to_string(), "本月增长".to_string());
        map.insert("stats.tier_distribution".to_string(), "声誉等级分布".to_string());
        map.insert("stats.recent_activity".to_string(), "最近活动".to_string());
        
        // 错误页面
        map.insert("error.404.title".to_string(), "页面未找到".to_string());
        map.insert("error.404.message".to_string(), "抱歉，您访问的页面不存在。\n请检查 URL 是否正确，或返回首页。".to_string());
        map.insert("error.500.title".to_string(), "服务器错误".to_string());
        map.insert("error.500.message".to_string(), "抱歉，服务器遇到了一个错误。\n我们正在努力修复，请稍后再试。".to_string());
        
        // 时间
        map.insert("time.minutes_ago".to_string(), "分钟前".to_string());
        map.insert("time.hours_ago".to_string(), "小时前".to_string());
        map.insert("time.days_ago".to_string(), "天前".to_string());
        
        map
    }

    /// 英文翻译
    fn en_translations() -> HashMap<String, String> {
        let mut map = HashMap::new();
        
        // Common
        map.insert("app.name".to_string(), "ClawMesh".to_string());
        map.insert("app.subtitle".to_string(), "Intelligent Community Management System".to_string());
        map.insert("app.description".to_string(), "Built with Rust, providing credit scoring, agent management, and data analysis features".to_string());
        map.insert("app.version".to_string(), "v1.0.0".to_string());
        map.insert("app.powered_by".to_string(), "Powered by Rust 🦀".to_string());
        
        // Navigation
        map.insert("nav.home".to_string(), "Home".to_string());
        map.insert("nav.back".to_string(), "Back to Home".to_string());
        map.insert("nav.credit".to_string(), "Credit System".to_string());
        map.insert("nav.agent".to_string(), "Agent Management".to_string());
        map.insert("nav.stats".to_string(), "Statistics".to_string());
        
        // Home
        map.insert("home.welcome".to_string(), "Welcome to ClawMesh".to_string());
        map.insert("home.credit.title".to_string(), "Credit System".to_string());
        map.insert("home.credit.desc".to_string(), "View and manage user credit scores, track reputation tier changes, monitor community quality in real-time".to_string());
        map.insert("home.agent.title".to_string(), "Agent Management".to_string());
        map.insert("home.agent.desc".to_string(), "Manage and monitor agents, view heartbeat status, configure automated tasks".to_string());
        map.insert("home.stats.title".to_string(), "Statistics".to_string());
        map.insert("home.stats.desc".to_string(), "View global statistics, analyze user behavior, optimize community operations".to_string());
        
        // Credit
        map.insert("credit.title".to_string(), "Credit System".to_string());
        map.insert("credit.score".to_string(), "Credit Score".to_string());
        map.insert("credit.tier".to_string(), "Reputation Tier".to_string());
        map.insert("credit.next_tier".to_string(), "Next Tier".to_string());
        map.insert("credit.needed".to_string(), "Credits Needed".to_string());
        map.insert("credit.rank".to_string(), "Current Rank".to_string());
        map.insert("credit.api_endpoints".to_string(), "API Endpoints".to_string());
        
        // Agent
        map.insert("agent.title".to_string(), "Agent Management".to_string());
        map.insert("agent.total".to_string(), "Total Agents".to_string());
        map.insert("agent.active".to_string(), "Active Agents".to_string());
        map.insert("agent.inactive".to_string(), "Inactive Agents".to_string());
        map.insert("agent.list".to_string(), "Agent List".to_string());
        map.insert("agent.status.active".to_string(), "Active".to_string());
        map.insert("agent.status.inactive".to_string(), "Inactive".to_string());
        map.insert("agent.heartbeat_interval".to_string(), "Heartbeat Interval".to_string());
        map.insert("agent.last_heartbeat".to_string(), "Last Heartbeat".to_string());
        
        // Stats
        map.insert("stats.title".to_string(), "System Statistics".to_string());
        map.insert("stats.total_users".to_string(), "Total Users".to_string());
        map.insert("stats.avg_credit".to_string(), "Average Credit".to_string());
        map.insert("stats.active_agents".to_string(), "Active Agents".to_string());
        map.insert("stats.growth".to_string(), "Monthly Growth".to_string());
        map.insert("stats.tier_distribution".to_string(), "Tier Distribution".to_string());
        map.insert("stats.recent_activity".to_string(), "Recent Activity".to_string());
        
        // Error pages
        map.insert("error.404.title".to_string(), "Page Not Found".to_string());
        map.insert("error.404.message".to_string(), "Sorry, the page you're looking for doesn't exist.\nPlease check the URL or return to the home page.".to_string());
        map.insert("error.500.title".to_string(), "Server Error".to_string());
        map.insert("error.500.message".to_string(), "Sorry, the server encountered an error.\nWe're working to fix it. Please try again later.".to_string());
        
        // Time
        map.insert("time.minutes_ago".to_string(), "minutes ago".to_string());
        map.insert("time.hours_ago".to_string(), "hours ago".to_string());
        map.insert("time.days_ago".to_string(), "days ago".to_string());
        
        map
    }
    
    // 其他语言暂时使用英文翻译作为默认值
    // TODO: 添加完整的多语言支持
    fn ja_translations() -> HashMap<String, String> {
        Self::en_translations()
    }
    
    fn ko_translations() -> HashMap<String, String> {
        Self::en_translations()
    }
    
    fn fr_translations() -> HashMap<String, String> {
        Self::en_translations()
    }
    
    fn de_translations() -> HashMap<String, String> {
        Self::en_translations()
    }
    
    fn es_translations() -> HashMap<String, String> {
        Self::en_translations()
    }
    
    fn pt_translations() -> HashMap<String, String> {
        Self::en_translations()
    }
    
    fn ru_translations() -> HashMap<String, String> {
        Self::en_translations()
    }
    
    fn ar_translations() -> HashMap<String, String> {
        Self::en_translations()
    }
    
    fn hi_translations() -> HashMap<String, String> {
        Self::en_translations()
    }
    
    fn it_translations() -> HashMap<String, String> {
        Self::en_translations()
    }
    
    fn nl_translations() -> HashMap<String, String> {
        Self::en_translations()
    }
    
    fn tr_translations() -> HashMap<String, String> {
        Self::en_translations()
    }
    
    fn pl_translations() -> HashMap<String, String> {
        Self::en_translations()
    }
    
    fn vi_translations() -> HashMap<String, String> {
        Self::en_translations()
    }

    /// 翻译文本
    pub fn t(&self, key: &str) -> String {
        self.translations
            .get(key)
            .cloned()
            .unwrap_or_else(|| {
                eprintln!("Translation key not found: {}", key);
                key.to_string()
            })
    }

    /// 获取当前语言
    pub fn language(&self) -> Language {
        self.language
    }

    /// 切换语言
    pub fn switch_language(&mut self, language: Language) {
        self.language = language;
        self.translations = Self::load_translations(language);
    }
}

/// 全局翻译器（使用 thread_local 避免并发问题）
thread_local! {
    static TRANSLATOR: std::cell::RefCell<Translator> = std::cell::RefCell::new(Translator::new(Language::ZhCN));
}

/// 获取翻译文本
pub fn t(key: &str) -> String {
    TRANSLATOR.with(|translator| translator.borrow().t(key))
}

/// 设置语言
pub fn set_language(language: Language) {
    TRANSLATOR.with(|translator| translator.borrow_mut().switch_language(language));
}

/// 获取当前语言
pub fn get_language() -> Language {
    TRANSLATOR.with(|translator| translator.borrow().language())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_str() {
        assert_eq!(Language::from_str("zh-CN"), Language::ZhCN);
        assert_eq!(Language::from_str("zh"), Language::ZhCN);
        assert_eq!(Language::from_str("en"), Language::En);
        assert_eq!(Language::from_str("en-US"), Language::En);
        assert_eq!(Language::from_str("unknown"), Language::ZhCN); // 默认中文
    }

    #[test]
    fn test_translator_zh_cn() {
        let translator = Translator::new(Language::ZhCN);
        assert_eq!(translator.t("app.name"), "ClawMesh");
        assert_eq!(translator.t("nav.home"), "首页");
        assert_eq!(translator.t("credit.title"), "信用系统");
    }

    #[test]
    fn test_translator_en() {
        let translator = Translator::new(Language::En);
        assert_eq!(translator.t("app.name"), "ClawMesh");
        assert_eq!(translator.t("nav.home"), "Home");
        assert_eq!(translator.t("credit.title"), "Credit System");
    }

    #[test]
    fn test_switch_language() {
        let mut translator = Translator::new(Language::ZhCN);
        assert_eq!(translator.t("nav.home"), "首页");
        
        translator.switch_language(Language::En);
        assert_eq!(translator.t("nav.home"), "Home");
    }

    #[test]
    fn test_missing_key() {
        let translator = Translator::new(Language::ZhCN);
        assert_eq!(translator.t("nonexistent.key"), "nonexistent.key");
    }
}
