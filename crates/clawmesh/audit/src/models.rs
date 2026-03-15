/// 审计日志数据模型

use serde::{Deserialize, Serialize};

/// 审计日志查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogQuery {
    /// 事件类型过滤
    pub event_type: Option<String>,
    /// 执行者 ID
    pub actor_id: Option<i32>,
    /// 目标 ID
    pub target_id: Option<i32>,
    /// 开始时间
    pub start_time: Option<String>,
    /// 结束时间
    pub end_time: Option<String>,
    /// 分页：页码
    pub page: Option<i32>,
    /// 分页：每页数量
    pub per_page: Option<i32>,
}

impl Default for AuditLogQuery {
    fn default() -> Self {
        Self {
            event_type: None,
            actor_id: None,
            target_id: None,
            start_time: None,
            end_time: None,
            page: Some(1),
            per_page: Some(50),
        }
    }
}
