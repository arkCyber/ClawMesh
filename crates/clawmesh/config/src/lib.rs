/// ClawMesh 配置管理系统
/// 
/// 提供动态配置加载和管理

use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

/// 全局配置实例
pub static CONFIG: Lazy<RwLock<ClawMeshConfig>> = Lazy::new(|| {
    RwLock::new(ClawMeshConfig::default())
});

/// ClawMesh 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClawMeshConfig {
    /// 信用系统配置
    pub credit: CreditConfig,
    /// 智能体系统配置
    pub agent: AgentConfig,
    /// 调度器配置
    pub scheduler: SchedulerConfig,
}

/// 信用系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditConfig {
    /// 帖子点赞信用
    pub post_upvote: i32,
    /// 帖子点踩信用
    pub post_downvote: i32,
    /// 评论点赞信用
    pub comment_upvote: i32,
    /// 评论点踩信用
    pub comment_downvote: i32,
    /// 每日活跃信用
    pub daily_active: i32,
    /// 最小发帖信用
    pub min_credit_to_post: i32,
    /// 最小创建社区信用
    pub min_credit_to_create_community: i32,
    /// 最小审核信用
    pub min_credit_to_moderate: i32,
    /// 最大信用分数
    pub max_credit: i32,
    /// 最小信用分数
    pub min_credit: i32,
}

/// 智能体系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// 最小心跳间隔（秒）
    pub min_heartbeat_interval: i32,
    /// 最大心跳间隔（秒）
    pub max_heartbeat_interval: i32,
    /// 默认心跳间隔（秒）
    pub default_heartbeat_interval: i32,
    /// 智能体初始信用
    pub initial_credit: i32,
    /// 最大元数据大小（字节）
    pub max_metadata_size: usize,
}

/// 调度器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// 智能体检查间隔（秒）
    pub agent_check_interval: u64,
    /// 数据清理间隔（秒）
    pub cleanup_interval: u64,
    /// 统计更新间隔（秒）
    pub stats_update_interval: u64,
}

impl Default for ClawMeshConfig {
    fn default() -> Self {
        Self {
            credit: CreditConfig::default(),
            agent: AgentConfig::default(),
            scheduler: SchedulerConfig::default(),
        }
    }
}

impl Default for CreditConfig {
    fn default() -> Self {
        Self {
            post_upvote: 2,
            post_downvote: -3,
            comment_upvote: 1,
            comment_downvote: -2,
            daily_active: 5,
            min_credit_to_post: 50,
            min_credit_to_create_community: 300,
            min_credit_to_moderate: 500,
            max_credit: 1000,
            min_credit: 0,
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            min_heartbeat_interval: 300,      // 5分钟
            max_heartbeat_interval: 86400,    // 24小时
            default_heartbeat_interval: 3600, // 1小时
            initial_credit: 300,
            max_metadata_size: 10240,         // 10KB
        }
    }
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            agent_check_interval: 3600,   // 1小时
            cleanup_interval: 86400,       // 24小时
            stats_update_interval: 1800,   // 30分钟
        }
    }
}

/// 加载配置
pub fn load_config() -> Result<ClawMeshConfig> {
    // 从环境变量或配置文件加载
    // 这里使用默认配置
    Ok(ClawMeshConfig::default())
}

/// 获取配置
pub fn get_config() -> ClawMeshConfig {
    CONFIG.read().unwrap().clone()
}

/// 更新配置
pub fn update_config(config: ClawMeshConfig) -> Result<()> {
    *CONFIG.write().unwrap() = config;
    Ok(())
}

/// 从 JSON 字符串加载配置
pub fn load_from_json(json: &str) -> Result<ClawMeshConfig> {
    let config: ClawMeshConfig = serde_json::from_str(json)?;
    Ok(config)
}

/// 将配置导出为 JSON
pub fn export_to_json(config: &ClawMeshConfig) -> Result<String> {
    let json = serde_json::to_string_pretty(config)?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ClawMeshConfig::default();
        assert_eq!(config.credit.post_upvote, 2);
        assert_eq!(config.credit.post_downvote, -3);
        assert_eq!(config.agent.initial_credit, 300);
    }

    #[test]
    fn test_json_serialization() {
        let config = ClawMeshConfig::default();
        let json = export_to_json(&config).unwrap();
        let loaded = load_from_json(&json).unwrap();
        
        assert_eq!(config.credit.post_upvote, loaded.credit.post_upvote);
        assert_eq!(config.agent.initial_credit, loaded.agent.initial_credit);
    }

    #[test]
    fn test_config_update() {
        let mut config = ClawMeshConfig::default();
        config.credit.post_upvote = 5;
        
        update_config(config.clone()).unwrap();
        let loaded = get_config();
        
        assert_eq!(loaded.credit.post_upvote, 5);
    }
}
