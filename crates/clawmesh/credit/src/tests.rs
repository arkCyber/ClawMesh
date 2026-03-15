#[cfg(test)]
mod tests {
    use crate::tier::ReputationTier;
    use crate::calculator::CreditAction;

    #[test]
    fn test_reputation_tiers() {
        use crate::tier::get_reputation_tier;
        
        assert_eq!(get_reputation_tier(50), ReputationTier::Novice);
        assert_eq!(get_reputation_tier(300), ReputationTier::Regular);
        assert_eq!(get_reputation_tier(600), ReputationTier::Active);
        assert_eq!(get_reputation_tier(800), ReputationTier::Veteran);
        assert_eq!(get_reputation_tier(900), ReputationTier::Expert);
    }

    #[test]
    fn test_credit_calculation() {
        use crate::calculator::calculate_credit_change;
        
        // Test credit action values
        assert_eq!(calculate_credit_change(&CreditAction::PostUpvote), 2);
        assert_eq!(calculate_credit_change(&CreditAction::CommentUpvote), 1);
        assert_eq!(calculate_credit_change(&CreditAction::PostDownvote), -3);
        assert_eq!(calculate_credit_change(&CreditAction::CommentDownvote), -2);
        assert_eq!(calculate_credit_change(&CreditAction::DailyActive), 5);
    }

    #[test]
    fn test_credit_score_bounds() {
        let score = 1500;
        let clamped = score.clamp(0, 1000);
        assert_eq!(clamped, 1000);

        let score = -100;
        let clamped = score.clamp(0, 1000);
        assert_eq!(clamped, 0);
    }

    #[test]
    fn test_tier_transitions() {
        use crate::tier::get_reputation_tier;
        
        // Test boundary conditions
        assert_eq!(get_reputation_tier(200), ReputationTier::Novice);
        assert_eq!(get_reputation_tier(201), ReputationTier::Regular);
        assert_eq!(get_reputation_tier(500), ReputationTier::Regular);
        assert_eq!(get_reputation_tier(501), ReputationTier::Active);
        assert_eq!(get_reputation_tier(700), ReputationTier::Active);
        assert_eq!(get_reputation_tier(701), ReputationTier::Veteran);
    }
}
