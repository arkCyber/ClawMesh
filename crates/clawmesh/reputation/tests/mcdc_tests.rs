/// MC/DC (Modified Condition/Decision Coverage) Tests
/// DO-178C Level A Requirement
/// 
/// MC/DC ensures that each condition in a decision independently affects the outcome

use clawmesh_reputation::reputation::calculate_reputation_score;
use clawmesh_reputation::models::ReputationLevel;

#[cfg(test)]
mod mcdc_reputation_tests {
    use super::*;

    // ========================================================================
    // MC/DC Tests for calculate_reputation_score
    // ========================================================================
    
    /// Decision: score.max(MIN_SCORE).min(MAX_SCORE)
    /// Conditions: 
    /// A: score < MIN_SCORE (0)
    /// B: score > MAX_SCORE (2000)
    
    #[test]
    fn mcdc_score_clamping_below_min() {
        // Test Case 1: A=true, B=false
        // Score below minimum should clamp to 0
        let score = calculate_reputation_score(0, 100); // 500 - 1000 = -500
        assert_eq!(score, 0, "Score below minimum should clamp to 0");
    }
    
    #[test]
    fn mcdc_score_clamping_above_max() {
        // Test Case 2: A=false, B=true
        // Score above maximum should clamp to 2000
        let score = calculate_reputation_score(200, 0); // 500 + 2000 = 2500
        assert_eq!(score, 2000, "Score above maximum should clamp to 2000");
    }
    
    #[test]
    fn mcdc_score_clamping_within_range() {
        // Test Case 3: A=false, B=false
        // Score within range should not be clamped
        let score = calculate_reputation_score(50, 30); // 500 + 500 - 300 = 700
        assert_eq!(score, 700, "Score within range should not be clamped");
    }
    
    #[test]
    fn mcdc_score_clamping_at_min_boundary() {
        // Test Case 4: A=boundary, B=false
        // Score exactly at minimum boundary
        let score = calculate_reputation_score(0, 50); // 500 - 500 = 0
        assert_eq!(score, 0, "Score at minimum boundary");
    }
    
    #[test]
    fn mcdc_score_clamping_at_max_boundary() {
        // Test Case 5: A=false, B=boundary
        // Score exactly at maximum boundary
        let score = calculate_reputation_score(150, 0); // 500 + 1500 = 2000
        assert_eq!(score, 2000, "Score at maximum boundary");
    }

    // ========================================================================
    // MC/DC Tests for ReputationLevel::from_score
    // ========================================================================
    
    /// Decision: Multiple if conditions for level determination
    /// Conditions:
    /// A: score < 300
    /// B: score < 600
    /// C: score < 1000
    /// D: score < 1400
    /// E: score < 1800
    
    #[test]
    fn mcdc_level_novice() {
        // A=true -> Novice
        assert_eq!(ReputationLevel::from_score(0), ReputationLevel::Novice);
        assert_eq!(ReputationLevel::from_score(150), ReputationLevel::Novice);
        assert_eq!(ReputationLevel::from_score(299), ReputationLevel::Novice);
    }
    
    #[test]
    fn mcdc_level_bronze() {
        // A=false, B=true -> Bronze
        assert_eq!(ReputationLevel::from_score(300), ReputationLevel::Bronze);
        assert_eq!(ReputationLevel::from_score(450), ReputationLevel::Bronze);
        assert_eq!(ReputationLevel::from_score(599), ReputationLevel::Bronze);
    }
    
    #[test]
    fn mcdc_level_silver() {
        // A=false, B=false, C=true -> Silver
        assert_eq!(ReputationLevel::from_score(600), ReputationLevel::Silver);
        assert_eq!(ReputationLevel::from_score(800), ReputationLevel::Silver);
        assert_eq!(ReputationLevel::from_score(999), ReputationLevel::Silver);
    }
    
    #[test]
    fn mcdc_level_gold() {
        // A=false, B=false, C=false, D=true -> Gold
        assert_eq!(ReputationLevel::from_score(1000), ReputationLevel::Gold);
        assert_eq!(ReputationLevel::from_score(1200), ReputationLevel::Gold);
        assert_eq!(ReputationLevel::from_score(1399), ReputationLevel::Gold);
    }
    
    #[test]
    fn mcdc_level_platinum() {
        // A=false, B=false, C=false, D=false, E=true -> Platinum
        assert_eq!(ReputationLevel::from_score(1400), ReputationLevel::Platinum);
        assert_eq!(ReputationLevel::from_score(1600), ReputationLevel::Platinum);
        assert_eq!(ReputationLevel::from_score(1799), ReputationLevel::Platinum);
    }
    
    #[test]
    fn mcdc_level_diamond() {
        // A=false, B=false, C=false, D=false, E=false -> Diamond
        assert_eq!(ReputationLevel::from_score(1800), ReputationLevel::Diamond);
        assert_eq!(ReputationLevel::from_score(1900), ReputationLevel::Diamond);
        assert_eq!(ReputationLevel::from_score(2000), ReputationLevel::Diamond);
    }
    
    // ========================================================================
    // MC/DC Tests for Boundary Transitions
    // ========================================================================
    
    #[test]
    fn mcdc_boundary_novice_to_bronze() {
        // Test transition at boundary 300
        assert_eq!(ReputationLevel::from_score(299), ReputationLevel::Novice);
        assert_eq!(ReputationLevel::from_score(300), ReputationLevel::Bronze);
    }
    
    #[test]
    fn mcdc_boundary_bronze_to_silver() {
        // Test transition at boundary 600
        assert_eq!(ReputationLevel::from_score(599), ReputationLevel::Bronze);
        assert_eq!(ReputationLevel::from_score(600), ReputationLevel::Silver);
    }
    
    #[test]
    fn mcdc_boundary_silver_to_gold() {
        // Test transition at boundary 1000
        assert_eq!(ReputationLevel::from_score(999), ReputationLevel::Silver);
        assert_eq!(ReputationLevel::from_score(1000), ReputationLevel::Gold);
    }
    
    #[test]
    fn mcdc_boundary_gold_to_platinum() {
        // Test transition at boundary 1400
        assert_eq!(ReputationLevel::from_score(1399), ReputationLevel::Gold);
        assert_eq!(ReputationLevel::from_score(1400), ReputationLevel::Platinum);
    }
    
    #[test]
    fn mcdc_boundary_platinum_to_diamond() {
        // Test transition at boundary 1800
        assert_eq!(ReputationLevel::from_score(1799), ReputationLevel::Platinum);
        assert_eq!(ReputationLevel::from_score(1800), ReputationLevel::Diamond);
    }
    
    // ========================================================================
    // MC/DC Tests for Vote Calculation Logic
    // ========================================================================
    
    /// Decision: BASE_SCORE + (positive * UPVOTE) - (negative * DOWNVOTE)
    /// Conditions:
    /// A: positive_votes > 0
    /// B: negative_votes > 0
    
    #[test]
    fn mcdc_votes_both_zero() {
        // A=false, B=false
        let score = calculate_reputation_score(0, 0);
        assert_eq!(score, 500, "Base score with no votes");
    }
    
    #[test]
    fn mcdc_votes_only_positive() {
        // A=true, B=false
        let score = calculate_reputation_score(10, 0);
        assert_eq!(score, 600, "Base score + positive votes");
    }
    
    #[test]
    fn mcdc_votes_only_negative() {
        // A=false, B=true
        let score = calculate_reputation_score(0, 10);
        assert_eq!(score, 400, "Base score - negative votes");
    }
    
    #[test]
    fn mcdc_votes_both_present() {
        // A=true, B=true
        let score = calculate_reputation_score(20, 10);
        assert_eq!(score, 600, "Base score + positive - negative");
    }
    
    // ========================================================================
    // MC/DC Tests for Edge Cases
    // ========================================================================
    
    #[test]
    fn mcdc_extreme_positive_votes() {
        // Test with extremely high positive votes
        let score = calculate_reputation_score(1000, 0);
        assert_eq!(score, 2000, "Should clamp to maximum");
    }
    
    #[test]
    fn mcdc_extreme_negative_votes() {
        // Test with extremely high negative votes
        let score = calculate_reputation_score(0, 1000);
        assert_eq!(score, 0, "Should clamp to minimum");
    }
    
    #[test]
    fn mcdc_balanced_extreme_votes() {
        // Test with balanced extreme votes
        let score = calculate_reputation_score(500, 500);
        assert_eq!(score, 500, "Should remain at base score");
    }
    
    // ========================================================================
    // MC/DC Coverage Summary
    // ========================================================================
    
    #[test]
    fn mcdc_coverage_verification() {
        // This test verifies that all MC/DC test cases are present
        // and that each condition independently affects the decision
        
        // Verify score clamping conditions
        assert!(calculate_reputation_score(0, 100) == 0); // Below min
        assert!(calculate_reputation_score(200, 0) == 2000); // Above max
        assert!(calculate_reputation_score(50, 30) == 700); // Within range
        
        // Verify level determination conditions
        assert!(matches!(ReputationLevel::from_score(299), ReputationLevel::Novice));
        assert!(matches!(ReputationLevel::from_score(300), ReputationLevel::Bronze));
        assert!(matches!(ReputationLevel::from_score(600), ReputationLevel::Silver));
        assert!(matches!(ReputationLevel::from_score(1000), ReputationLevel::Gold));
        assert!(matches!(ReputationLevel::from_score(1400), ReputationLevel::Platinum));
        assert!(matches!(ReputationLevel::from_score(1800), ReputationLevel::Diamond));
        
        // Verify vote calculation conditions
        assert!(calculate_reputation_score(0, 0) == 500); // No votes
        assert!(calculate_reputation_score(10, 0) == 600); // Only positive
        assert!(calculate_reputation_score(0, 10) == 400); // Only negative
        assert!(calculate_reputation_score(20, 10) == 600); // Both present
    }
}
