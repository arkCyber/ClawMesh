use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInstallRequest {
    pub username: String,
    pub agent_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInstallResponse {
    pub person_id: i32,
    pub username: String,
    pub credit_score: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub person_id: i32,
    pub last_heartbeat: DateTime<Utc>,
    pub heartbeat_interval: i32,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreditResponse {
    pub person_id: i32,
    pub username: String,
    pub credit_score: i32,
    pub reputation_tier: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreditHistoryResponse {
    pub person_id: i32,
    pub username: String,
    pub credit_score: i32,
    pub reputation_tier: String,
    pub history: Vec<CreditHistoryItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreditHistoryItem {
    pub action_type: String,
    pub credit_change: i32,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}
