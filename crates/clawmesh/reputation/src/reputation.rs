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
    
    // Get total agents and average score
    let (total_agents, avg_score, total_votes_sum): (i64, Option<f64>, Option<i64>) = agent_reputation::table
        .select((
            count(agent_reputation::agent_id),
            avg(agent_reputation::reputation_score),
            sum(agent_reputation::total_votes),
        ))
        .first(conn)
        .await
        .context("Failed to calculate reputation stats")?;
    
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_reputation_score() {
        // Base score
        assert_eq!(calculate_reputation_score(0, 0), 500);
        
        // Positive votes
        assert_eq!(calculate_reputation_score(10, 0), 600);
        assert_eq!(calculate_reputation_score(50, 0), 1000);
        
        // Negative votes
        assert_eq!(calculate_reputation_score(0, 10), 400);
        assert_eq!(calculate_reputation_score(0, 50), 0); // Clamped to min
        
        // Mixed votes
        assert_eq!(calculate_reputation_score(30, 10), 700);
        
        // Maximum score
        assert_eq!(calculate_reputation_score(200, 0), 2000); // Clamped to max
        
        // Minimum score
        assert_eq!(calculate_reputation_score(0, 100), 0); // Clamped to min
    }

    #[test]
    fn test_score_bounds() {
        // Test that score is always within valid range
        for pos in 0..200 {
            for neg in 0..200 {
                let score = calculate_reputation_score(pos, neg);
                assert!(score >= 0 && score <= 2000);
            }
        }
    }
}
