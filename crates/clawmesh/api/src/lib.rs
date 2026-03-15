pub mod agent;
pub mod agent_auth;
pub mod agent_list;
pub mod agent_reputation;
pub mod agent_skills;
pub mod agent_workspace;
pub mod agent_social;
pub mod agent_marketplace;
pub mod auth;
pub mod config;
pub mod credit;
pub mod direct_message;
pub mod error;
pub mod friendship;
pub mod graceful_shutdown;
pub mod health;
pub mod lemmy_extensions;
pub mod metrics;
pub mod permissions;
pub mod rate_limit;
pub mod rate_limiter;
pub mod responses;
pub mod routes;
pub mod stats;

// Use Lemmy's authentication system instead of custom JWT
pub use lemmy_api_utils::{
    claims::Claims,
    local_user_view_from_jwt,
    local_user_view_from_jwt_opt,
};
pub use lemmy_db_views_local_user::LocalUserView;

// ClawMesh extensions to Lemmy
pub use lemmy_extensions::{
    ExtendedUserInfo,
    get_extended_user_from_jwt,
    require_extended_user,
    require_credit_score,
    require_mod_or_admin,
};

pub use auth::{SecurityContext, UserRole, Permission, check_permission as check_auth_permission};
pub use config::{AppConfig, ServerConfig, DatabaseConfig, WebSocketConfig, RateLimitConfig as RateLimitCfg, MonitoringConfig, ConfigError};
pub use error::{ClawMeshError, ClawMeshResult, ErrorCode};
pub use graceful_shutdown::{ShutdownCoordinator, ShutdownManager, ShutdownError, ShutdownPhase, setup_signal_handlers};
pub use health::{HealthChecker, HealthStatus, SystemHealth, ComponentHealth, liveness_probe, readiness_probe, health_check};
pub use metrics::{MetricsRegistry, ClawMeshMetrics};
pub use rate_limit::{RateLimiter, RateLimitAction, RateLimitConfig};
pub use rate_limiter::{InMemoryRateLimiter, RateLimiterMiddleware, GlobalRateLimiter};

pub use agent::{agent_install, get_agent_heartbeat, update_agent_heartbeat, get_skill, update_agent, update_agent_status, delete_agent};
pub use agent_auth::{generate_agent_token, refresh_agent_token, revoke_agent_token};
pub use agent_list::{get_agent_count, get_agent_details, get_stale_agents_list, list_all_agents};
pub use agent_reputation::{get_reputation, vote_for_agent, get_reputation_history, get_leaderboard, get_reputation_stats};
pub use agent_skills::{register_agent_skill, list_agent_skills, install_agent_skill, delete_agent_skill, execute_agent_skill, search_marketplace, get_marketplace_statistics, publish_to_marketplace};
pub use credit::{get_credit_history, get_user_credit};
pub use friendship::config_friendship_routes;
pub use permissions::check_permission;
pub use responses::*;
pub use routes::config;
pub use stats::{get_global_credit_stats, get_person_credit_stats};
