// Additional comprehensive tests for credit module
#[cfg(test)]
mod credit_integration_tests {
    use super::*;

    #[test]
    fn test_credit_action_values() {
        use crate::calculator::CreditAction;
        
        // Verify all credit action values
        assert_eq!(CreditAction::PostUpvote.value(), 2);
        assert_eq!(CreditAction::PostDownvote.value(), -3);
        assert_eq!(CreditAction::CommentUpvote.value(), 1);
        assert_eq!(CreditAction::CommentDownvote.value(), -2);
        assert_eq!(CreditAction::DailyActive.value(), 5);
    }

    #[test]
    fn test_reputation_tier_ordering() {
        use crate::tier::{ReputationTier, get_reputation_tier};
        
        // Test that tiers are ordered correctly
        let novice = get_reputation_tier(100);
        let regular = get_reputation_tier(300);
        let active = get_reputation_tier(600);
        let veteran = get_reputation_tier(800);
        let expert = get_reputation_tier(900);
        
        assert!(matches!(novice, ReputationTier::Novice));
        assert!(matches!(regular, ReputationTier::Regular));
        assert!(matches!(active, ReputationTier::Active));
        assert!(matches!(veteran, ReputationTier::Veteran));
        assert!(matches!(expert, ReputationTier::Expert));
    }

    #[test]
    fn test_credit_score_clamping() {
        // Test that credit scores are properly bounded
        let max_score = 1000;
        let min_score = 0;
        
        // Test upper bound
        let over_max = 1500;
        assert_eq!(over_max.clamp(min_score, max_score), max_score);
        
        // Test lower bound
        let under_min = -100;
        assert_eq!(under_min.clamp(min_score, max_score), min_score);
        
        // Test within bounds
        let normal = 500;
        assert_eq!(normal.clamp(min_score, max_score), normal);
    }

    #[test]
    fn test_tier_boundaries() {
        use crate::tier::{ReputationTier, get_reputation_tier};
        
        // Test exact boundaries
        assert!(matches!(get_reputation_tier(0), ReputationTier::Novice));
        assert!(matches!(get_reputation_tier(200), ReputationTier::Novice));
        assert!(matches!(get_reputation_tier(201), ReputationTier::Regular));
        assert!(matches!(get_reputation_tier(500), ReputationTier::Regular));
        assert!(matches!(get_reputation_tier(501), ReputationTier::Active));
        assert!(matches!(get_reputation_tier(700), ReputationTier::Active));
        assert!(matches!(get_reputation_tier(701), ReputationTier::Veteran));
        assert!(matches!(get_reputation_tier(850), ReputationTier::Veteran));
        assert!(matches!(get_reputation_tier(851), ReputationTier::Expert));
        assert!(matches!(get_reputation_tier(1000), ReputationTier::Expert));
    }

    #[test]
    fn test_permission_thresholds() {
        use crate::permissions::{MIN_CREDIT_TO_POST, MIN_CREDIT_TO_MODERATE, MIN_CREDIT_TO_CREATE_COMMUNITY};
        
        // Verify permission thresholds are reasonable
        assert_eq!(MIN_CREDIT_TO_POST, 50);
        assert_eq!(MIN_CREDIT_TO_MODERATE, 500);
        assert_eq!(MIN_CREDIT_TO_CREATE_COMMUNITY, 300);
        
        // Verify ordering makes sense
        assert!(MIN_CREDIT_TO_POST < MIN_CREDIT_TO_CREATE_COMMUNITY);
        assert!(MIN_CREDIT_TO_CREATE_COMMUNITY < MIN_CREDIT_TO_MODERATE);
    }

    #[test]
    fn test_violation_severity() {
        use crate::calculator::ViolationSeverity;
        
        // Test violation penalties
        assert_eq!(ViolationSeverity::Minor.penalty(), -10);
        assert_eq!(ViolationSeverity::Moderate.penalty(), -25);
        assert_eq!(ViolationSeverity::Severe.penalty(), -50);
        assert_eq!(ViolationSeverity::Critical.penalty(), -100);
        
        // Verify penalties are ordered
        assert!(ViolationSeverity::Minor.penalty() > ViolationSeverity::Moderate.penalty());
        assert!(ViolationSeverity::Moderate.penalty() > ViolationSeverity::Severe.penalty());
        assert!(ViolationSeverity::Severe.penalty() > ViolationSeverity::Critical.penalty());
    }

    #[test]
    fn test_community_creation_credit() {
        use crate::calculator::calculate_community_creation_credit;
        
        // Test credit calculation for community creation
        let credit_1_member = calculate_community_creation_credit(1);
        let credit_10_members = calculate_community_creation_credit(10);
        let credit_100_members = calculate_community_creation_credit(100);
        
        // More members should give more credit
        assert!(credit_10_members > credit_1_member);
        assert!(credit_100_members > credit_10_members);
        
        // But should be capped
        let credit_1000_members = calculate_community_creation_credit(1000);
        let credit_10000_members = calculate_community_creation_credit(10000);
        assert_eq!(credit_1000_members, credit_10000_members);
    }
}
