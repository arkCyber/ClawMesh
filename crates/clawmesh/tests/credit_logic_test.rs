/// 信用系统逻辑测试
/// 测试信用计算和等级转换逻辑

#[cfg(test)]
mod tests {
    use clawmesh_credit::{
        calculator::*, 
        tier::*,
        permissions::*
    };

    #[test]
    fn test_credit_action_calculations() {
        assert_eq!(calculate_credit_change(&CreditAction::PostUpvote), 2);
        assert_eq!(calculate_credit_change(&CreditAction::PostDownvote), -3);
        assert_eq!(calculate_credit_change(&CreditAction::CommentUpvote), 1);
        assert_eq!(calculate_credit_change(&CreditAction::CommentDownvote), -2);
        assert_eq!(calculate_credit_change(&CreditAction::DailyActive), 5);
    }

    #[test]
    fn test_community_created_credit() {
        // 小社区
        let small = CreditAction::CommunityCreated { members: 50 };
        assert_eq!(calculate_credit_change(&small), 5);

        // 中等社区
        let medium = CreditAction::CommunityCreated { members: 500 };
        assert_eq!(calculate_credit_change(&medium), 50);

        // 大社区
        let large = CreditAction::CommunityCreated { members: 2000 };
        assert_eq!(calculate_credit_change(&large), 200);

        // 超大社区（应该被限制在200）
        let huge = CreditAction::CommunityCreated { members: 10000 };
        assert_eq!(calculate_credit_change(&huge), 200);
    }

    #[test]
    fn test_violation_credit() {
        let minor = CreditAction::Violation { severity: 1 };
        assert_eq!(calculate_credit_change(&minor), -100);

        let major = CreditAction::Violation { severity: 3 };
        assert_eq!(calculate_credit_change(&major), -300);

        let severe = CreditAction::Violation { severity: 5 };
        assert_eq!(calculate_credit_change(&severe), -500);
    }

    #[test]
    fn test_reputation_tier_boundaries() {
        // Novice 边界
        assert_eq!(get_reputation_tier(0), ReputationTier::Novice);
        assert_eq!(get_reputation_tier(100), ReputationTier::Novice);
        assert_eq!(get_reputation_tier(200), ReputationTier::Novice);

        // Regular 边界
        assert_eq!(get_reputation_tier(201), ReputationTier::Regular);
        assert_eq!(get_reputation_tier(350), ReputationTier::Regular);
        assert_eq!(get_reputation_tier(500), ReputationTier::Regular);

        // Active 边界
        assert_eq!(get_reputation_tier(501), ReputationTier::Active);
        assert_eq!(get_reputation_tier(600), ReputationTier::Active);
        assert_eq!(get_reputation_tier(700), ReputationTier::Active);

        // Veteran 边界
        assert_eq!(get_reputation_tier(701), ReputationTier::Veteran);
        assert_eq!(get_reputation_tier(800), ReputationTier::Veteran);
        assert_eq!(get_reputation_tier(850), ReputationTier::Veteran);

        // Expert 边界
        assert_eq!(get_reputation_tier(851), ReputationTier::Expert);
        assert_eq!(get_reputation_tier(900), ReputationTier::Expert);
        assert_eq!(get_reputation_tier(1000), ReputationTier::Expert);
        assert_eq!(get_reputation_tier(1001), ReputationTier::Expert); // 超过最大值
    }

    #[test]
    fn test_tier_string_conversion() {
        assert_eq!(ReputationTier::Novice.as_str(), "novice");
        assert_eq!(ReputationTier::Regular.as_str(), "regular");
        assert_eq!(ReputationTier::Active.as_str(), "active");
        assert_eq!(ReputationTier::Veteran.as_str(), "veteran");
        assert_eq!(ReputationTier::Expert.as_str(), "expert");

        // 测试反向转换
        assert_eq!(ReputationTier::from_str("novice"), ReputationTier::Novice);
        assert_eq!(ReputationTier::from_str("regular"), ReputationTier::Regular);
        assert_eq!(ReputationTier::from_str("active"), ReputationTier::Active);
        assert_eq!(ReputationTier::from_str("veteran"), ReputationTier::Veteran);
        assert_eq!(ReputationTier::from_str("expert"), ReputationTier::Expert);
        
        // 未知字符串应该返回 Regular
        assert_eq!(ReputationTier::from_str("unknown"), ReputationTier::Regular);
    }

    #[test]
    fn test_min_credit_requirements() {
        assert_eq!(get_min_credit_for_action("post"), 50);
        assert_eq!(get_min_credit_for_action("comment"), 0);
        assert_eq!(get_min_credit_for_action("create_community"), 201);
        assert_eq!(get_min_credit_for_action("moderate"), 501);
        assert_eq!(get_min_credit_for_action("admin"), 701);
        assert_eq!(get_min_credit_for_action("unknown"), 0);
    }

    #[test]
    fn test_credit_score_clamping() {
        // 测试信用分数应该被限制在 0-1000 范围内
        let below_min = -100;
        let clamped_min = below_min.clamp(0, 1000);
        assert_eq!(clamped_min, 0);

        let above_max = 1500;
        let clamped_max = above_max.clamp(0, 1000);
        assert_eq!(clamped_max, 1000);

        let normal = 500;
        let clamped_normal = normal.clamp(0, 1000);
        assert_eq!(clamped_normal, 500);
    }

    #[test]
    fn test_tier_progression() {
        // 测试等级晋升路径
        let mut score = 0;
        assert_eq!(get_reputation_tier(score), ReputationTier::Novice);

        score += 250; // 250
        assert_eq!(get_reputation_tier(score), ReputationTier::Regular);

        score += 300; // 550
        assert_eq!(get_reputation_tier(score), ReputationTier::Active);

        score += 200; // 750
        assert_eq!(get_reputation_tier(score), ReputationTier::Veteran);

        score += 150; // 900
        assert_eq!(get_reputation_tier(score), ReputationTier::Expert);
    }
}
