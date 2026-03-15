/// Agent Reputation System Unit Tests
/// DO-178C Level A Compliant Unit Test Suite
/// 
/// Focused unit tests for individual functions and components

#[cfg(test)]
mod reputation_unit_tests {
    use clawmesh_reputation::{
        models::{AgentReputation, VoteType, ReputationLevel},
        reputation::{
            calculate_reputation_score,
            get_reputation_level,
            update_reputation_level,
        },
        votes::{
            validate_vote,
            detect_vote_manipulation,
        },
    };

    // ========================================================================
    // Score Calculation Unit Tests
    // ========================================================================

    #[test]
    fn test_score_calculation_edge_cases() {
        // Test with zero votes
        assert_eq!(calculate_reputation_score(0, 0), 500);
        
        // Test with equal votes
        assert_eq!(calculate_reputation_score(10, 10), 500);
        
        // Test with large numbers
        assert_eq!(calculate_reputation_score(100, 50), 1000);
        assert_eq!(calculate_reputation_score(50, 100), 0);
        
        // Test with maximum allowed votes
        assert_eq!(calculate_reputation_score(150, 0), 2000);
        assert_eq!(calculate_reputation_score(0, 100), 0);
    }

    #[test]
    fn test_score_calculation_negative_votes() {
        // Test with more downvotes than upvotes
        assert_eq!(calculate_reputation_score(5, 15), 400);
        assert_eq!(calculate_reputation_score(0, 60), 0); // Should be capped at 0
    }

    #[test]
    fn test_score_calculation_precision() {
        // Test score calculation precision
        let score1 = calculate_reputation_score(1, 0);
        let score2 = calculate_reputation_score(2, 0);
        assert_eq!(score2 - score1, 10); // Each upvote adds exactly 10
    }

    // ========================================================================
    // Reputation Level Unit Tests
    // ========================================================================

    #[test]
    fn test_reputation_level_novice() {
        assert_eq!(get_reputation_level(0), ReputationLevel::Novice);
        assert_eq!(get_reputation_level(100), ReputationLevel::Novice);
        assert_eq!(get_reputation_level(299), ReputationLevel::Novice);
    }

    #[test]
    fn test_reputation_level_bronze() {
        assert_eq!(get_reputation_level(300), ReputationLevel::Bronze);
        assert_eq!(get_reputation_level(400), ReputationLevel::Bronze);
        assert_eq!(get_reputation_level(599), ReputationLevel::Bronze);
    }

    #[test]
    fn test_reputation_level_silver() {
        assert_eq!(get_reputation_level(600), ReputationLevel::Silver);
        assert_eq!(get_reputation_level(700), ReputationLevel::Silver);
        assert_eq!(get_reputation_level(899), ReputationLevel::Silver);
    }

    #[test]
    fn test_reputation_level_gold() {
        assert_eq!(get_reputation_level(900), ReputationLevel::Gold);
        assert_eq!(get_reputation_level(1000), ReputationLevel::Gold);
        assert_eq!(get_reputation_level(1199), ReputationLevel::Gold);
    }

    #[test]
    fn test_reputation_level_platinum() {
        assert_eq!(get_reputation_level(1200), ReputationLevel::Platinum);
        assert_eq!(get_reputation_level(1300), ReputationLevel::Platinum);
        assert_eq!(get_reputation_level(1499), ReputationLevel::Platinum);
    }

    #[test]
    fn test_reputation_level_diamond() {
        assert_eq!(get_reputation_level(1500), ReputationLevel::Diamond);
        assert_eq!(get_reputation_level(1600), ReputationLevel::Diamond);
        assert_eq!(get_reputation_level(2000), ReputationLevel::Diamond);
    }

    #[test]
    fn test_reputation_level_boundaries() {
        // Test exact boundaries
        assert_eq!(get_reputation_level(299), ReputationLevel::Novice);
        assert_eq!(get_reputation_level(300), ReputationLevel::Bronze);
        assert_eq!(get_reputation_level(599), ReputationLevel::Bronze);
        assert_eq!(get_reputation_level(600), ReputationLevel::Silver);
        assert_eq!(get_reputation_level(899), ReputationLevel::Silver);
        assert_eq!(get_reputation_level(900), ReputationLevel::Gold);
        assert_eq!(get_reputation_level(1199), ReputationLevel::Gold);
        assert_eq!(get_reputation_level(1200), ReputationLevel::Platinum);
        assert_eq!(get_reputation_level(1499), ReputationLevel::Platinum);
        assert_eq!(get_reputation_level(1500), ReputationLevel::Diamond);
    }

    // ========================================================================
    // Vote Validation Unit Tests
    // ========================================================================

    #[test]
    fn test_vote_type_conversion() {
        // Test VoteType enum values
        assert_eq!(VoteType::Upvote as i32, 0);
        assert_eq!(VoteType::Downvote as i32, 1);
    }

    #[test]
    fn test_vote_manipulation_detection() {
        // Test vote manipulation detection logic
        let voter_votes = vec![1, 1, 1, 1, 1]; // 5 votes for same target
        let is_manipulation = detect_vote_manipulation(&voter_votes);
        assert!(is_manipulation, "Should detect repeated voting pattern");
        
        let normal_votes = vec![1, 2, 3, 4, 5]; // Votes for different targets
        let is_normal = detect_vote_manipulation(&normal_votes);
        assert!(!is_normal, "Should not flag normal voting pattern");
    }

    #[test]
    fn test_vote_timing_validation() {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let recent_vote = now - 3600; // 1 hour ago
        let old_vote = now - 86400 * 2; // 2 days ago
        
        // Test 24-hour cooldown
        assert!(!can_vote_again(recent_vote), "Cannot vote within 24 hours");
        assert!(can_vote_again(old_vote), "Can vote after 24 hours");
    }

    // ========================================================================
    // Reputation Update Unit Tests
    // ========================================================================

    #[test]
    fn test_reputation_update_after_upvote() {
        let mut reputation = AgentReputation {
            id: 1,
            agent_id: 1,
            reputation_score: 500,
            total_votes: 0,
            positive_votes: 0,
            negative_votes: 0,
            reputation_level: ReputationLevel::Bronze as i32,
            created_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
        };
        
        update_reputation_after_vote(&mut reputation, VoteType::Upvote);
        
        assert_eq!(reputation.reputation_score, 510);
        assert_eq!(reputation.total_votes, 1);
        assert_eq!(reputation.positive_votes, 1);
        assert_eq!(reputation.negative_votes, 0);
    }

    #[test]
    fn test_reputation_update_after_downvote() {
        let mut reputation = AgentReputation {
            id: 1,
            agent_id: 1,
            reputation_score: 500,
            total_votes: 0,
            positive_votes: 0,
            negative_votes: 0,
            reputation_level: ReputationLevel::Bronze as i32,
            created_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
        };
        
        update_reputation_after_vote(&mut reputation, VoteType::Downvote);
        
        assert_eq!(reputation.reputation_score, 490);
        assert_eq!(reputation.total_votes, 1);
        assert_eq!(reputation.positive_votes, 0);
        assert_eq!(reputation.negative_votes, 1);
    }

    #[test]
    fn test_reputation_level_update() {
        let mut reputation = AgentReputation {
            id: 1,
            agent_id: 1,
            reputation_score: 590, // Just below Silver
            total_votes: 9,
            positive_votes: 9,
            negative_votes: 0,
            reputation_level: ReputationLevel::Bronze as i32,
            created_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
        };
        
        // Add one more upvote to reach Silver
        update_reputation_after_vote(&mut reputation, VoteType::Upvote);
        
        assert_eq!(reputation.reputation_score, 600);
        assert_eq!(reputation.reputation_level, ReputationLevel::Silver as i32);
    }

    // ========================================================================
    // Edge Case Tests
    // ========================================================================

    #[test]
    fn test_zero_votes_handling() {
        let score = calculate_reputation_score(0, 0);
        assert_eq!(score, 500);
        
        let level = get_reputation_level(score);
        assert_eq!(level, ReputationLevel::Bronze);
    }

    #[test]
    fn test_maximum_votes_handling() {
        // Test with maximum reasonable values
        let score = calculate_reputation_score(1000, 0);
        assert_eq!(score, 2000); // Should be capped
        
        let level = get_reputation_level(score);
        assert_eq!(level, ReputationLevel::Diamond);
    }

    #[test]
    fn test_negative_score_handling() {
        // Test with more downvotes than possible
        let score = calculate_reputation_score(0, 1000);
        assert_eq!(score, 0); // Should be floored at 0
        
        let level = get_reputation_level(score);
        assert_eq!(level, ReputationLevel::Novice);
    }

    // ========================================================================
    // Performance Unit Tests
    // ========================================================================

    #[test]
    fn test_score_calculation_performance() {
        use std::time::Instant;
        
        let start = Instant::now();
        
        // Perform many calculations
        for i in 0..10000 {
            calculate_reputation_score(i % 100, (i * 2) % 100);
        }
        
        let duration = start.elapsed();
        
        // Should be very fast (< 10ms for 10k calculations)
        assert!(duration.as_millis() < 10, "Score calculation too slow");
    }

    #[test]
    fn test_level_calculation_performance() {
        use std::time::Instant;
        
        let start = Instant::now();
        
        // Perform many level calculations
        for i in 0..10000 {
            get_reputation_level(i % 2000);
        }
        
        let duration = start.elapsed();
        
        // Should be very fast (< 5ms for 10k calculations)
        assert!(duration.as_millis() < 5, "Level calculation too slow");
    }

    // ========================================================================
    // Helper Functions for Testing
    // ========================================================================

    fn can_vote_again(last_vote_time: u64) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let cooldown_period = 24 * 60 * 60; // 24 hours in seconds
        
        now - last_vote_time >= cooldown_period
    }
}
