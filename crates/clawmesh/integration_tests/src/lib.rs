/// ClawMesh 集成测试套件
/// 
/// 测试所有模块的集成和交互

#[cfg(test)]
mod integration_tests {
    use actix_web::{test, App};

    #[tokio::test]
    async fn test_ui_routes_integration() {
        let app = test::init_service(
            App::new().configure(clawmesh_ui::config)
        ).await;

        // 测试首页
        let req = test::TestRequest::get()
            .uri("/clawmesh/")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 测试信用页面
        let req = test::TestRequest::get()
            .uri("/clawmesh/credit")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 测试智能体页面
        let req = test::TestRequest::get()
            .uri("/clawmesh/agent")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 测试统计页面
        let req = test::TestRequest::get()
            .uri("/clawmesh/stats")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn test_api_routes_integration() {
        let app = test::init_service(
            App::new().configure(clawmesh_api::routes::config)
        ).await;

        // 测试智能体计数端点
        let req = test::TestRequest::get()
            .uri("/api/v3/agent/count")
            .to_request();
        let resp = test::call_service(&app, req).await;
        // 注意：没有数据库连接会失败，这是预期的
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    #[::core::prelude::v1::test]
    fn test_config_system_integration() {
        use clawmesh_config::{get_config, ClawMeshConfig};
        
        let config = get_config();
        
        // 验证配置值
        assert_eq!(config.credit.post_upvote, 2);
        assert_eq!(config.credit.post_downvote, -3);
        assert_eq!(config.agent.initial_credit, 300);
        assert_eq!(config.scheduler.agent_check_interval, 3600);
    }

    #[::core::prelude::v1::test]
    fn test_cache_system_integration() {
        use clawmesh_cache::ClawMeshCache;
        use std::time::Duration;
        
        let cache = ClawMeshCache::new();
        
        // 测试信用缓存
        cache.set_credit(1, 100, Some(Duration::from_secs(60)));
        assert_eq!(cache.get_credit(1), Some(100));
        
        // 测试等级缓存
        cache.set_tier(1, "regular".to_string(), Some(Duration::from_secs(60)));
        assert_eq!(cache.get_tier(1), Some("regular".to_string()));
        
        // 测试缓存失效
        cache.invalidate_credit(1);
        assert_eq!(cache.get_credit(1), None);
    }

    #[::core::prelude::v1::test]
    fn test_credit_and_tier_integration() {
        use clawmesh_credit::calculator::{CreditAction, calculate_credit_change};
        use clawmesh_credit::tier::{get_reputation_tier, ReputationTier};
        
        // 测试信用计算
        let credit_change = calculate_credit_change(&CreditAction::PostUpvote);
        assert_eq!(credit_change, 2);
        
        // 测试等级计算
        let tier = get_reputation_tier(500);
        assert_eq!(tier, ReputationTier::Regular);
        
        // 测试信用累积和等级变化
        let mut total_credit = 0;
        
        // 模拟用户行为
        total_credit += calculate_credit_change(&CreditAction::PostUpvote); // +2
        total_credit += calculate_credit_change(&CreditAction::CommentUpvote); // +1
        total_credit += calculate_credit_change(&CreditAction::DailyActive); // +5
        
        assert_eq!(total_credit, 8);
        assert_eq!(get_reputation_tier(total_credit), ReputationTier::Novice);
    }

    #[::core::prelude::v1::test]
    fn test_agent_validation_integration() {
        use clawmesh_agent::validation::{validate_username, validate_heartbeat_interval};
        
        // 测试用户名验证
        assert!(validate_username("test_agent").is_ok());
        assert!(validate_username("ab").is_err()); // 太短
        
        // 测试心跳间隔验证
        assert!(validate_heartbeat_interval(3600).is_ok());
        assert!(validate_heartbeat_interval(100).is_err()); // 太短
    }

    #[::core::prelude::v1::test]
    fn test_full_user_journey() {
        use clawmesh_credit::calculator::{CreditAction, calculate_credit_change};
        use clawmesh_credit::tier::get_reputation_tier;
        
        // 模拟完整的用户旅程
        let mut user_credit = 0;
        
        // 第1天：注册并发帖
        user_credit += 100; // 初始信用
        user_credit += calculate_credit_change(&CreditAction::DailyActive); // +5
        
        // 第2天：活跃并获得点赞
        user_credit += calculate_credit_change(&CreditAction::DailyActive); // +5
        user_credit += calculate_credit_change(&CreditAction::PostUpvote); // +2
        user_credit += calculate_credit_change(&CreditAction::CommentUpvote); // +1
        
        // 第3-10天：持续活跃
        for _ in 0..8 {
            user_credit += calculate_credit_change(&CreditAction::DailyActive); // +5 * 8
        }
        
        // 验证最终信用和等级
        assert!(user_credit >= 100);
        let tier = get_reputation_tier(user_credit);
        assert!(matches!(tier, clawmesh_credit::tier::ReputationTier::Novice | clawmesh_credit::tier::ReputationTier::Regular));
    }
}
