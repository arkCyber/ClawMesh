/// ClawMesh 定时任务调度器
/// 
/// 定期执行维护任务

use anyhow::Result;
use diesel_async::AsyncPgConnection;
use std::time::Duration;
use tokio::time;
use tracing::{error, info};

pub mod tasks;

/// 定时任务配置
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// 智能体活跃度检查间隔（秒）
    pub agent_check_interval: u64,
    /// 数据清理间隔（秒）
    pub cleanup_interval: u64,
    /// 统计更新间隔（秒）
    pub stats_update_interval: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            agent_check_interval: 3600,      // 1小时
            cleanup_interval: 86400,          // 24小时
            stats_update_interval: 1800,      // 30分钟
        }
    }
}

/// 启动定时任务调度器
pub async fn start_scheduler(config: SchedulerConfig) {
    info!("Starting ClawMesh scheduler with config: {:?}", config);

    // 启动智能体活跃度检查任务
    let agent_config = config.clone();
    tokio::spawn(async move {
        run_agent_check_task(agent_config.agent_check_interval).await;
    });

    // 启动数据清理任务
    let cleanup_config = config.clone();
    tokio::spawn(async move {
        run_cleanup_task(cleanup_config.cleanup_interval).await;
    });

    // 启动统计更新任务
    let stats_config = config.clone();
    tokio::spawn(async move {
        run_stats_update_task(stats_config.stats_update_interval).await;
    });

    info!("ClawMesh scheduler started successfully");
}

/// 运行智能体活跃度检查任务
async fn run_agent_check_task(interval_secs: u64) {
    let mut interval = time::interval(Duration::from_secs(interval_secs));

    loop {
        interval.tick().await;
        
        info!("Running agent activity check task");
        
        // 注意：这里需要数据库连接，实际使用时需要从连接池获取
        // 这里只是框架，实际实现需要传入连接池
        if let Err(e) = tasks::check_agent_activity().await {
            error!("Agent activity check failed: {}", e);
        }
    }
}

/// 运行数据清理任务
async fn run_cleanup_task(interval_secs: u64) {
    let mut interval = time::interval(Duration::from_secs(interval_secs));

    loop {
        interval.tick().await;
        
        info!("Running data cleanup task");
        
        if let Err(e) = tasks::cleanup_old_data().await {
            error!("Data cleanup failed: {}", e);
        }
    }
}

/// 运行统计更新任务
async fn run_stats_update_task(interval_secs: u64) {
    let mut interval = time::interval(Duration::from_secs(interval_secs));

    loop {
        interval.tick().await;
        
        info!("Running stats update task");
        
        if let Err(e) = tasks::update_statistics().await {
            error!("Stats update failed: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SchedulerConfig::default();
        assert_eq!(config.agent_check_interval, 3600);
        assert_eq!(config.cleanup_interval, 86400);
        assert_eq!(config.stats_update_interval, 1800);
    }
}
