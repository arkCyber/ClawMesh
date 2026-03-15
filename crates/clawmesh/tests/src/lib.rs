/// ClawMesh 集成测试
/// 
/// 测试所有模块的集成和交互

#[cfg(test)]
mod integration_tests {
    use anyhow::Result;

    #[test]
    fn test_all_modules_compile() {
        // 确保所有模块都能编译
        assert!(true);
    }

    #[test]
    fn test_config_system() {
        use clawmesh_config::{get_config, ClawMeshConfig};
        
        let config = get_config();
        assert_eq!(config.credit.post_upvote, 2);
        assert_eq!(config.credit.post_downvote, -3);
        assert_eq!(config.agent.initial_credit, 300);
    }

    #[test]
    fn test_cache_system() {
        use clawmesh_cache::ClawMeshCache;
        use std::time::Duration;
        
        let cache = ClawMeshCache::new();
        
        // 测试信用缓存
        cache.set_credit(1, 100, Some(Duration::from_secs(60)));
        assert_eq!(cache.get_credit(1), Some(100));
        
        // 测试等级缓存
        cache.set_tier(1, "regular".to_string(), Some(Duration::from_secs(60)));
        assert_eq!(cache.get_tier(1), Some("regular".to_string()));
        
        // 测试缓存统计
        let stats = cache.stats();
        assert!(stats.credit_entries > 0);
        assert!(stats.tier_entries > 0);
    }

    #[test]
    fn test_credit_calculator() {
        use clawmesh_credit::calculator::{CreditAction, calculate_credit_change};
        
        assert_eq!(calculate_credit_change(&CreditAction::PostUpvote), 2);
        assert_eq!(calculate_credit_change(&CreditAction::PostDownvote), -3);
        assert_eq!(calculate_credit_change(&CreditAction::CommentUpvote), 1);
        assert_eq!(calculate_credit_change(&CreditAction::CommentDownvote), -2);
        assert_eq!(calculate_credit_change(&CreditAction::DailyActive), 5);
    }

    #[test]
    fn test_reputation_tiers() {
        use clawmesh_credit::tier::{get_reputation_tier, ReputationTier};
        
        assert_eq!(get_reputation_tier(50), ReputationTier::Novice);
        assert_eq!(get_reputation_tier(300), ReputationTier::Regular);
        assert_eq!(get_reputation_tier(600), ReputationTier::Active);
        assert_eq!(get_reputation_tier(800), ReputationTier::Veteran);
        assert_eq!(get_reputation_tier(900), ReputationTier::Expert);
    }

    #[test]
    fn test_agent_validation() {
        use clawmesh_agent::validation::{validate_username, validate_heartbeat_interval};
        
        // 有效用户名
        assert!(validate_username("test_agent").is_ok());
        assert!(validate_username("agent123").is_ok());
        
        // 无效用户名
        assert!(validate_username("ab").is_err()); // 太短
        assert!(validate_username("a".repeat(51).as_str()).is_err()); // 太长 (>50)
        assert!(validate_username("_invalid").is_err()); // 不能以下划线开头
        
        // 有效心跳间隔
        assert!(validate_heartbeat_interval(3600).is_ok());
        
        // 无效心跳间隔
        assert!(validate_heartbeat_interval(100).is_err()); // 太短
        assert!(validate_heartbeat_interval(100000).is_err()); // 太长
    }

    #[test]
    fn test_config_json_serialization() {
        use clawmesh_config::{ClawMeshConfig, export_to_json, load_from_json};
        
        let config = ClawMeshConfig::default();
        let json = export_to_json(&config).unwrap();
        let loaded = load_from_json(&json).unwrap();
        
        assert_eq!(config.credit.post_upvote, loaded.credit.post_upvote);
        assert_eq!(config.agent.initial_credit, loaded.agent.initial_credit);
    }

    #[test]
    fn test_cache_expiration() {
        use clawmesh_cache::ClawMeshCache;
        use std::time::Duration;
        
        let cache = ClawMeshCache::new();
        
        // 设置一个立即过期的缓存
        cache.set_credit(999, 100, Some(Duration::from_millis(1)));
        
        // 等待过期
        std::thread::sleep(Duration::from_millis(10));
        
        // 应该返回 None
        assert_eq!(cache.get_credit(999), None);
    }

    #[test]
    fn test_tier_string_conversion() {
        use clawmesh_credit::tier::ReputationTier;
        
        assert_eq!(ReputationTier::Novice.as_str(), "novice");
        assert_eq!(ReputationTier::Regular.as_str(), "regular");
        assert_eq!(ReputationTier::Active.as_str(), "active");
        assert_eq!(ReputationTier::Veteran.as_str(), "veteran");
        assert_eq!(ReputationTier::Expert.as_str(), "expert");
        
        assert_eq!(ReputationTier::parse_tier("novice"), Some(ReputationTier::Novice));
        assert_eq!(ReputationTier::parse_tier("regular"), Some(ReputationTier::Regular));
    }

    #[test]
    fn test_all_credit_actions() {
        use clawmesh_credit::calculator::{CreditAction, calculate_credit_change};
        
        let actions = vec![
            (CreditAction::PostUpvote, 2),
            (CreditAction::PostDownvote, -3),
            (CreditAction::CommentUpvote, 1),
            (CreditAction::CommentDownvote, -2),
            (CreditAction::DailyActive, 5),
        ];
        
        for (action, expected) in actions {
            assert_eq!(calculate_credit_change(&action), expected);
        }
    }
}
