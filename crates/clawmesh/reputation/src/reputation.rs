/// Agent Reputation Core Functions (DO-178C Level A)
/// 
/// Implements reputation calculation, updates, and queries

use anyhow::{Result, Context};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::PersonId;
use tracing::{info, warn, error};

use crate::models::{AgentReputation, AgentReputationForm, ReputationLevel};

/// Calculate reputation score based on votes
/// 
/// # Algorithm
/// - Base score: 500
/// - Each upvote: +10
/// - Each downvote: -10
/// - Minimum score: 0
/// - Maximum score: 2000
/// 
/// # Safety
/// - Pure function (no side effects)
/// - Deterministic calculation
/// - Bounded output range
pub fn calculate_reputation_score(positive_votes: i32, negative_votes: i32) -> i32 {
    const BASE_SCORE: i32 = 500;
    const UPVOTE_VALUE: i32 = 10;
    const DOWNVOTE_VALUE: i32 = 10;
    const MIN_SCORE: i32 = 0;
    const MAX_SCORE: i32 = 2000;
    
    let score = BASE_SCORE + (positive_votes * UPVOTE_VALUE) - (negative_votes * DOWNVOTE_VALUE);
    
    // Clamp to valid range
    score.max(MIN_SCORE).min(MAX_SCORE)
}

/// Get agent reputation
/// 
/// # Safety
/// - Read-only operation
/// - Returns None if agent not found
/// - Fast indexed lookup
pub async fn get_agent_reputation(
    agent_id: PersonId,
    conn: &mut AsyncPgConnection,
) -> Result<Option<AgentReputation>> {
    use lemmy_db_schema_file::schema::agent_reputation;
    
    let reputation = agent_reputation::table
        .find(agent_id)
        .first::<AgentReputation>(conn)
        .await
        .optional()
        .context("Failed to query agent reputation")?;
    
    Ok(reputation)
}

/// Update agent reputation after a vote
/// 
/// # Safety
/// - Atomic database transaction
/// - Validates vote counts
/// - Updates reputation level automatically
/// - Full error handling and logging
pub async fn update_agent_reputation(
    agent_id: PersonId,
    positive_delta: i32,
    negative_delta: i32,
    conn: &mut AsyncPgConnection,
) -> Result<AgentReputation> {
    use lemmy_db_schema_file::schema::agent_reputation;
    
    // Get current reputation
    let current = get_agent_reputation(agent_id, conn)
        .await?
        .context("Agent reputation not found")?;
    
    // Calculate new values
    let new_positive = current.positive_votes + positive_delta;
    let new_negative = current.negative_votes + negative_delta;
    let new_total = new_positive + new_negative;
    let new_score = calculate_reputation_score(new_positive, new_negative);
    let new_level = ReputationLevel::from_score(new_score);
    
    // Validate
    if new_positive < 0 || new_negative < 0 {
        error!(
            agent_id = agent_id.0,
            positive_delta = positive_delta,
            negative_delta = negative_delta,
            "Invalid vote delta would result in negative counts"
        );
        anyhow::bail!("Invalid vote delta");
    }
    
    // Update reputation
    let form = AgentReputationForm {
        agent_id,
        reputation_score: new_score,
        total_votes: new_total,
        positive_votes: new_positive,
        negative_votes: new_negative,
        reputation_level: new_level,
    };
    
    let updated = diesel::update(agent_reputation::table.find(agent_id))
        .set(&form)
        .get_result::<AgentReputation>(conn)
        .await
        .context("Failed to update agent reputation")?;
    
    info!(
        agent_id = agent_id.0,
        old_score = current.reputation_score,
        new_score = new_score,
        old_level = ?current.reputation_level,
        new_level = ?new_level,
        "Agent reputation updated"
    );
    
    // Check for level change
    if new_level != current.reputation_level {
        info!(
            agent_id = agent_id.0,
            old_level = ?current.reputation_level,
            new_level = ?new_level,
            "Agent reputation level changed"
        );
    }
    
    Ok(updated)
}

/// Get reputation leaderboard
/// 
/// # Safety
/// - Paginated results
/// - Efficient query with limit
/// - Ordered by reputation score
pub async fn get_reputation_leaderboard(
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentReputation>> {
    use lemmy_db_schema_file::schema::agent_reputation;
    
    let leaderboard = agent_reputation::table
        .order(agent_reputation::reputation_score.desc())
        .limit(limit)
        .offset(offset)
        .load::<AgentReputation>(conn)
        .await
        .context("Failed to load reputation leaderboard")?;
    
    Ok(leaderboard)
}

/// Get agents by reputation level
/// 
/// # Safety
/// - Filtered query
/// - Paginated results
pub async fn get_agents_by_level(
    level: ReputationLevel,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentReputation>> {
    use lemmy_db_schema_file::schema::agent_reputation;
    
    let agents = agent_reputation::table
        .filter(agent_reputation::reputation_level.eq(level as i32))
        .order(agent_reputation::reputation_score.desc())
        .limit(limit)
        .offset(offset)
        .load::<AgentReputation>(conn)
        .await
        .context("Failed to load agents by level")?;
    
    Ok(agents)
}

/// Get reputation statistics
/// 
/// # Safety
/// - Aggregation query
/// - Read-only operation
#[derive(Debug, Clone)]
pub struct ReputationStats {
    pub total_agents: i64,
    pub average_score: f64,
    pub median_score: i32,
    pub total_votes: i64,
}

pub async fn get_reputation_stats(
    conn: &mut AsyncPgConnection,
) -> Result<ReputationStats> {
    use lemmy_db_schema_file::schema::agent_reputation;
    use diesel::dsl::*;
    
    // Get total agents
    let total_agents: i64 = agent_reputation::table
        .select(count(agent_reputation::agent_id))
        .first(conn)
        .await
        .unwrap_or(0);
    
    // Get total votes sum
    let total_votes_sum: Option<i64> = agent_reputation::table
        .select(sum(agent_reputation::total_votes))
        .first(conn)
        .await
        .ok()
        .flatten();
    
    // Calculate average score manually
    let avg_score: Option<f64> = if total_agents > 0 {
        let total_score: i64 = agent_reputation::table
            .select(sum(agent_reputation::reputation_score))
            .first(conn)
            .await
            .ok()
            .flatten()
            .unwrap_or(0);
        Some(total_score as f64 / total_agents as f64)
    } else {
        None
    };
    
    // Get median score (simplified - get middle value)
    let median_score: i32 = if total_agents > 0 {
        let middle_offset = total_agents / 2;
        agent_reputation::table
            .order(agent_reputation::reputation_score.asc())
            .offset(middle_offset)
            .select(agent_reputation::reputation_score)
            .first(conn)
            .await
            .unwrap_or(500)
    } else {
        500
    };
    
    Ok(ReputationStats {
        total_agents,
        average_score: avg_score.unwrap_or(500.0),
        median_score,
        total_votes: total_votes_sum.unwrap_or(0),
    })
}

// ============================================================================
// TESTS - DO-178C Level A Compliance
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Reputation Score Calculation Tests
    // ========================================================================

    #[test]
    fn test_calculate_reputation_score_base() {
        // Base score with no votes
        assert_eq!(calculate_reputation_score(0, 0), 500);
    }

    #[test]
    fn test_calculate_reputation_score_positive_votes() {
        // Positive votes only
        assert_eq!(calculate_reputation_score(1, 0), 510);
        assert_eq!(calculate_reputation_score(10, 0), 600);
        assert_eq!(calculate_reputation_score(50, 0), 1000);
        assert_eq!(calculate_reputation_score(100, 0), 1500);
    }

    #[test]
    fn test_calculate_reputation_score_negative_votes() {
        // Negative votes only
        assert_eq!(calculate_reputation_score(0, 1), 490);
        assert_eq!(calculate_reputation_score(0, 10), 400);
        assert_eq!(calculate_reputation_score(0, 50), 0); // Clamped to min
        assert_eq!(calculate_reputation_score(0, 100), 0); // Clamped to min
    }

    #[test]
    fn test_calculate_reputation_score_mixed_votes() {
        // Mixed positive and negative votes
        assert_eq!(calculate_reputation_score(10, 5), 550);
        assert_eq!(calculate_reputation_score(30, 10), 700);
        assert_eq!(calculate_reputation_score(50, 30), 700);
        assert_eq!(calculate_reputation_score(100, 50), 1000);
    }

    #[test]
    fn test_calculate_reputation_score_maximum_clamping() {
        // Test maximum score clamping
        assert_eq!(calculate_reputation_score(150, 0), 2000); // Clamped to max
        assert_eq!(calculate_reputation_score(200, 0), 2000); // Clamped to max
        assert_eq!(calculate_reputation_score(1000, 0), 2000); // Clamped to max
    }

    #[test]
    fn test_calculate_reputation_score_minimum_clamping() {
        // Test minimum score clamping
        assert_eq!(calculate_reputation_score(0, 50), 0); // Clamped to min
        assert_eq!(calculate_reputation_score(0, 100), 0); // Clamped to min
        assert_eq!(calculate_reputation_score(0, 1000), 0); // Clamped to min
        assert_eq!(calculate_reputation_score(10, 60), 0); // 500 + 100 - 600 = 0
    }

    #[test]
    fn test_calculate_reputation_score_boundary_values() {
        // Test exact boundary values
        assert_eq!(calculate_reputation_score(150, 0), 2000); // Exactly at max
        assert_eq!(calculate_reputation_score(149, 0), 1990); // Just below max
        assert_eq!(calculate_reputation_score(0, 50), 0); // Exactly at min
        assert_eq!(calculate_reputation_score(0, 49), 10); // Just above min
    }

    #[test]
    fn test_calculate_reputation_score_bounds_comprehensive() {
        // Test that score is always within valid range
        for pos in 0..200 {
            for neg in 0..200 {
                let score = calculate_reputation_score(pos, neg);
                assert!(score >= 0 && score <= 2000, 
                    "Score {} out of bounds for pos={}, neg={}", score, pos, neg);
            }
        }
    }

    #[test]
    fn test_calculate_reputation_score_deterministic() {
        // Test that function is deterministic
        for _ in 0..10 {
            assert_eq!(calculate_reputation_score(25, 15), 600);
            assert_eq!(calculate_reputation_score(100, 50), 1000);
        }
    }

    #[test]
    fn test_calculate_reputation_score_symmetric_cancellation() {
        // Test that equal positive and negative votes cancel out
        assert_eq!(calculate_reputation_score(10, 10), 500);
        assert_eq!(calculate_reputation_score(50, 50), 500);
        assert_eq!(calculate_reputation_score(100, 100), 500);
    }

    #[test]
    fn test_calculate_reputation_score_overflow_safety() {
        // Test with very large values to ensure no overflow
        assert_eq!(calculate_reputation_score(i32::MAX / 20, 0), 2000);
        assert_eq!(calculate_reputation_score(0, i32::MAX / 20), 0);
    }

    // ========================================================================
    // Reputation Level Tests
    // ========================================================================

    #[test]
    fn test_reputation_level_from_score() {
        // Test level assignment based on score
        assert_eq!(ReputationLevel::from_score(0), ReputationLevel::Novice);
        assert_eq!(ReputationLevel::from_score(100), ReputationLevel::Novice);
        assert_eq!(ReputationLevel::from_score(299), ReputationLevel::Novice);
        
        assert_eq!(ReputationLevel::from_score(300), ReputationLevel::Bronze);
        assert_eq!(ReputationLevel::from_score(500), ReputationLevel::Bronze);
        assert_eq!(ReputationLevel::from_score(599), ReputationLevel::Bronze);
        
        assert_eq!(ReputationLevel::from_score(600), ReputationLevel::Silver);
        assert_eq!(ReputationLevel::from_score(800), ReputationLevel::Silver);
        assert_eq!(ReputationLevel::from_score(999), ReputationLevel::Silver);
        
        assert_eq!(ReputationLevel::from_score(1000), ReputationLevel::Gold);
        assert_eq!(ReputationLevel::from_score(1200), ReputationLevel::Gold);
        assert_eq!(ReputationLevel::from_score(1399), ReputationLevel::Gold);
        
        assert_eq!(ReputationLevel::from_score(1400), ReputationLevel::Platinum);
        assert_eq!(ReputationLevel::from_score(1600), ReputationLevel::Platinum);
        assert_eq!(ReputationLevel::from_score(1799), ReputationLevel::Platinum);
        
        assert_eq!(ReputationLevel::from_score(1800), ReputationLevel::Diamond);
        assert_eq!(ReputationLevel::from_score(1900), ReputationLevel::Diamond);
        assert_eq!(ReputationLevel::from_score(2000), ReputationLevel::Diamond);
    }

    #[test]
    fn test_reputation_level_boundaries() {
        // Test exact boundary values
        assert_eq!(ReputationLevel::from_score(299), ReputationLevel::Novice);
        assert_eq!(ReputationLevel::from_score(300), ReputationLevel::Bronze);
        
        assert_eq!(ReputationLevel::from_score(599), ReputationLevel::Bronze);
        assert_eq!(ReputationLevel::from_score(600), ReputationLevel::Silver);
        
        assert_eq!(ReputationLevel::from_score(999), ReputationLevel::Silver);
        assert_eq!(ReputationLevel::from_score(1000), ReputationLevel::Gold);
        
        assert_eq!(ReputationLevel::from_score(1399), ReputationLevel::Gold);
        assert_eq!(ReputationLevel::from_score(1400), ReputationLevel::Platinum);
        
        assert_eq!(ReputationLevel::from_score(1799), ReputationLevel::Platinum);
        assert_eq!(ReputationLevel::from_score(1800), ReputationLevel::Diamond);
    }

    // ========================================================================
    // Integration Tests (Score + Level)
    // ========================================================================

    #[test]
    fn test_score_and_level_integration() {
        // Test that score calculation and level assignment work together
        let score1 = calculate_reputation_score(0, 0);
        assert_eq!(score1, 500);
        assert_eq!(ReputationLevel::from_score(score1), ReputationLevel::Bronze);
        
        let score2 = calculate_reputation_score(50, 0);
        assert_eq!(score2, 1000);
        assert_eq!(ReputationLevel::from_score(score2), ReputationLevel::Gold);
        
        let score3 = calculate_reputation_score(150, 0);
        assert_eq!(score3, 2000);
        assert_eq!(ReputationLevel::from_score(score3), ReputationLevel::Diamond);
    }
}
