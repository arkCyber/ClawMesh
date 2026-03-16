/// Agent Reputation Data Models (DO-178C Level A)
/// 
/// Defines data structures for reputation management

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::{AsExpression, FromSqlRow};
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, ToSql, Output};
use diesel::sql_types::Integer;
use diesel::pg::{Pg, PgValue};
use lemmy_db_schema_file::PersonId;
use serde::{Deserialize, Serialize};

/// Reputation levels based on score ranges
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[derive(diesel::AsExpression, diesel::FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Integer)]
pub enum ReputationLevel {
    Novice = 0,      // 0-299
    Bronze = 1,      // 300-599
    Silver = 2,      // 600-899
    Gold = 3,        // 900-1199
    Platinum = 4,    // 1200-1499
    Diamond = 5,     // 1500+
}

impl ReputationLevel {
    /// Calculate reputation level from score
    pub fn from_score(score: i32) -> Self {
        match score {
            s if s < 300 => ReputationLevel::Novice,
            s if s < 600 => ReputationLevel::Bronze,
            s if s < 900 => ReputationLevel::Silver,
            s if s < 1200 => ReputationLevel::Gold,
            s if s < 1500 => ReputationLevel::Platinum,
            _ => ReputationLevel::Diamond,
        }
    }
    
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(ReputationLevel::Novice),
            1 => Some(ReputationLevel::Bronze),
            2 => Some(ReputationLevel::Silver),
            3 => Some(ReputationLevel::Gold),
            4 => Some(ReputationLevel::Platinum),
            5 => Some(ReputationLevel::Diamond),
            _ => None,
        }
    }
    
    /// Get minimum score for this level
    pub fn min_score(&self) -> i32 {
        match self {
            ReputationLevel::Novice => 0,
            ReputationLevel::Bronze => 300,
            ReputationLevel::Silver => 600,
            ReputationLevel::Gold => 900,
            ReputationLevel::Platinum => 1200,
            ReputationLevel::Diamond => 1500,
        }
    }
    
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            ReputationLevel::Novice => "Novice",
            ReputationLevel::Bronze => "Bronze",
            ReputationLevel::Silver => "Silver",
            ReputationLevel::Gold => "Gold",
            ReputationLevel::Platinum => "Platinum",
            ReputationLevel::Diamond => "Diamond",
        }
    }
}

// Implement ToSql for ReputationLevel
impl diesel::serialize::ToSql<diesel::sql_types::Integer, diesel::pg::Pg> for ReputationLevel {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>) -> diesel::serialize::Result {
        let value = *self as i32;
        <i32 as diesel::serialize::ToSql<diesel::sql_types::Integer, diesel::pg::Pg>>::to_sql(&value, &mut out.reborrow())
    }
}

// Implement FromSql for ReputationLevel
impl diesel::deserialize::FromSql<diesel::sql_types::Integer, diesel::pg::Pg> for ReputationLevel {
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        let value = <i32 as diesel::deserialize::FromSql<diesel::sql_types::Integer, diesel::pg::Pg>>::from_sql(bytes)?;
        Self::from_i32(value).ok_or_else(|| format!("Invalid ReputationLevel value: {}", value).into())
    }
}

/// Agent reputation record
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = agent_reputation)]
#[diesel(primary_key(agent_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentReputation {
    pub agent_id: PersonId,
    pub reputation_score: i32,
    pub total_votes: i32,
    pub positive_votes: i32,
    pub negative_votes: i32,
    pub reputation_level: ReputationLevel,
    pub last_updated: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl AgentReputation {
    /// Calculate reputation percentage (0-100)
    pub fn reputation_percentage(&self) -> f64 {
        if self.total_votes == 0 {
            50.0 // Neutral
        } else {
            (self.positive_votes as f64 / self.total_votes as f64) * 100.0
        }
    }
    
    /// Check if reputation is positive
    pub fn is_positive(&self) -> bool {
        self.reputation_score >= 500
    }
    
    /// Check if reputation is excellent
    pub fn is_excellent(&self) -> bool {
        self.reputation_score >= 900 && self.reputation_percentage() >= 80.0
    }
}

/// Form for creating/updating agent reputation
#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = agent_reputation)]
pub struct AgentReputationForm {
    pub agent_id: PersonId,
    pub reputation_score: i32,
    pub total_votes: i32,
    pub positive_votes: i32,
    pub negative_votes: i32,
    pub reputation_level: ReputationLevel,
}

/// Vote types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Integer)]
pub enum VoteType {
    Upvote,
    Downvote,
}

impl VoteType {
    /// Get score delta for this vote type
    pub fn score_delta(&self) -> i32 {
        match self {
            VoteType::Upvote => 10,
            VoteType::Downvote => -10,
        }
    }
    
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            VoteType::Upvote => "upvote",
            VoteType::Downvote => "downvote",
        }
    }
    
    /// Convert to i32 for database storage
    pub fn to_i32(&self) -> i32 {
        match self {
            VoteType::Upvote => 1,
            VoteType::Downvote => -1,
        }
    }
    
    /// Convert from i32
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(VoteType::Upvote),
            -1 => Some(VoteType::Downvote),
            _ => None,
        }
    }
}

// Implement ToSql for VoteType
impl diesel::serialize::ToSql<diesel::sql_types::Integer, diesel::pg::Pg> for VoteType {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        let value = self.to_i32();
        <i32 as diesel::serialize::ToSql<diesel::sql_types::Integer, diesel::pg::Pg>>::to_sql(&value, &mut out.reborrow())
    }
}

// Implement FromSql for VoteType
impl diesel::deserialize::FromSql<diesel::sql_types::Integer, diesel::pg::Pg> for VoteType {
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        let value = <i32 as diesel::deserialize::FromSql<diesel::sql_types::Integer, diesel::pg::Pg>>::from_sql(bytes)?;
        VoteType::from_i32(value)
            .ok_or_else(|| format!("Invalid VoteType value: {}", value).into())
    }
}

/// Reputation vote history record
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = agent_reputation_history)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentReputationHistory {
    pub id: i32,
    pub agent_id: PersonId,
    pub voter_id: PersonId,
    pub vote_type: VoteType,
    pub reason: Option<String>,
    pub score_before: i32,
    pub score_after: i32,
    pub created_at: DateTime<Utc>,
}

/// Form for creating reputation history
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = agent_reputation_history)]
pub struct AgentReputationHistoryForm {
    pub agent_id: PersonId,
    pub voter_id: PersonId,
    pub vote_type: VoteType,
    pub reason: Option<String>,
    pub score_before: i32,
    pub score_after: i32,
}

// Use schema from lemmy_db_schema_file
use lemmy_db_schema_file::schema::{agent_reputation, agent_reputation_history};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_level_from_score() {
        assert_eq!(ReputationLevel::from_score(0), ReputationLevel::Novice);
        assert_eq!(ReputationLevel::from_score(299), ReputationLevel::Novice);
        assert_eq!(ReputationLevel::from_score(300), ReputationLevel::Bronze);
        assert_eq!(ReputationLevel::from_score(599), ReputationLevel::Bronze);
        assert_eq!(ReputationLevel::from_score(600), ReputationLevel::Silver);
        assert_eq!(ReputationLevel::from_score(900), ReputationLevel::Gold);
        assert_eq!(ReputationLevel::from_score(1200), ReputationLevel::Platinum);
        assert_eq!(ReputationLevel::from_score(1500), ReputationLevel::Diamond);
        assert_eq!(ReputationLevel::from_score(2000), ReputationLevel::Diamond);
    }

    #[test]
    fn test_vote_type_score_delta() {
        assert_eq!(VoteType::Upvote.score_delta(), 10);
        assert_eq!(VoteType::Downvote.score_delta(), -10);
    }

    #[test]
    fn test_reputation_percentage() {
        let rep = AgentReputation {
            agent_id: PersonId(1),
            reputation_score: 500,
            total_votes: 100,
            positive_votes: 80,
            negative_votes: 20,
            reputation_level: ReputationLevel::Bronze,
            last_updated: Utc::now(),
            created_at: Utc::now(),
        };
        
        assert_eq!(rep.reputation_percentage(), 80.0);
        assert!(rep.is_positive());
    }
}
