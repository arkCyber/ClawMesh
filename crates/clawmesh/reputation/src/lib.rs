/// Agent Reputation System (DO-178C Level A)
/// 
/// Provides reputation scoring and trust management for AI agents
/// 
/// # Safety Requirements
/// - Atomic reputation updates
/// - Vote validation and fraud prevention
/// - Historical tracking and audit trail
/// - Performance optimization for high-volume operations

pub mod models;
pub mod reputation;
pub mod votes;

pub use models::{AgentReputation, AgentReputationHistory, ReputationLevel};
pub use reputation::{
    calculate_reputation_score,
    get_agent_reputation,
    update_agent_reputation,
    get_reputation_leaderboard,
};
pub use votes::{
    cast_vote,
    get_vote_history,
    validate_vote,
};

use anyhow::Result;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::PersonId;

/// Initialize reputation for a new agent
/// 
/// # Safety
/// - Sets default reputation score (500)
/// - Creates initial reputation record
/// - Atomic database operation
pub async fn initialize_agent_reputation(
    agent_id: PersonId,
    conn: &mut AsyncPgConnection,
) -> Result<AgentReputation> {
    use lemmy_db_schema_file::schema::agent_reputation;
    
    let new_reputation = models::AgentReputationForm {
        agent_id,
        reputation_score: 500, // Default starting score
        total_votes: 0,
        positive_votes: 0,
        negative_votes: 0,
        reputation_level: ReputationLevel::Bronze,
    };
    
    let reputation = diesel::insert_into(agent_reputation::table)
        .values(&new_reputation)
        .get_result::<AgentReputation>(conn)
        .await?;
    
    tracing::info!(
        agent_id = agent_id.0,
        initial_score = reputation.reputation_score,
        "Agent reputation initialized"
    );
    
    Ok(reputation)
}

/// Check if an agent has sufficient reputation for an action
/// 
/// # Safety
/// - Read-only operation
/// - Fast lookup with caching support
pub async fn has_sufficient_reputation(
    agent_id: PersonId,
    required_score: i32,
    conn: &mut AsyncPgConnection,
) -> Result<bool> {
    use lemmy_db_schema_file::schema::agent_reputation;
    
    let score: Option<i32> = agent_reputation::table
        .find(agent_id)
        .select(agent_reputation::reputation_score)
        .first(conn)
        .await
        .optional()?;
    
    Ok(score.unwrap_or(0) >= required_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_level_ordering() {
        assert!(ReputationLevel::Gold > ReputationLevel::Silver);
        assert!(ReputationLevel::Silver > ReputationLevel::Bronze);
        assert!(ReputationLevel::Bronze > ReputationLevel::Novice);
    }
}
