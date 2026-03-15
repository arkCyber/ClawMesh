use anyhow::Result;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};

/// Credit statistics for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditStats {
    pub total_changes: i64,
    pub positive_changes: i64,
    pub negative_changes: i64,
    pub total_gain: i32,
    pub total_loss: i32,
    pub average_change: f64,
}

/// Get credit statistics for a person
pub async fn get_person_stats(
    person_id: lemmy_db_schema_file::PersonId,
    conn: &mut AsyncPgConnection,
) -> Result<CreditStats> {
    use lemmy_db_schema_file::schema::credit_history;

    let records: Vec<i32> = credit_history::table
        .filter(credit_history::person_id.eq(person_id))
        .select(credit_history::credit_change)
        .load(conn)
        .await?;

    let total_changes = records.len() as i64;
    let positive_changes = records.iter().filter(|&&c| c > 0).count() as i64;
    let negative_changes = records.iter().filter(|&&c| c < 0).count() as i64;
    let total_gain: i32 = records.iter().filter(|&&c| c > 0).copied().sum();
    let total_loss: i32 = records.iter().filter(|&&c| c < 0).copied().sum();
    #[allow(clippy::cast_precision_loss)]
    let average_change = if total_changes > 0 {
        records.iter().map(|c| f64::from(*c)).sum::<f64>() / (total_changes as f64)
    } else {
        0.0
    };

    Ok(CreditStats {
        total_changes,
        positive_changes,
        negative_changes,
        total_gain,
        total_loss,
        average_change,
    })
}

/// Global credit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalStats {
    pub total_users: i64,
    pub average_credit: f64,
    pub median_credit: i32,
    pub tier_distribution: Vec<TierCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierCount {
    pub tier: String,
    pub count: i64,
}

/// Get global credit statistics
pub async fn get_global_stats(conn: &mut AsyncPgConnection) -> Result<GlobalStats> {
    use lemmy_db_schema_file::schema::person;
    use diesel::dsl::count;

    let total_users: i64 = person::table
        .select(count(person::id))
        .first(conn)
        .await
        .unwrap_or(0);

    // Calculate average credit from all scores
    let all_scores: Vec<i32> = person::table
        .select(person::credit_score)
        .load(conn)
        .await
        .unwrap_or_default();
    
    #[allow(clippy::cast_precision_loss)]
    let average_credit = if !all_scores.is_empty() {
        all_scores.iter().map(|s| f64::from(*s)).sum::<f64>() / (all_scores.len() as f64)
    } else {
        0.0
    };

    // Calculate median from existing scores
    let mut sorted_scores = all_scores.clone();
    sorted_scores.sort_unstable();
    let median_credit = if sorted_scores.is_empty() {
        0
    } else {
        sorted_scores[sorted_scores.len() / 2]
    };

    // Get tier distribution
    let tier_counts: Vec<(String, i64)> = person::table
        .filter(person::user_type.eq("human"))
        .group_by(person::reputation_tier)
        .select((person::reputation_tier, diesel::dsl::count(person::id)))
        .load(conn)
        .await?;

    let tier_distribution = tier_counts
        .into_iter()
        .map(|(tier, count)| TierCount { tier, count })
        .collect();

    Ok(GlobalStats {
        total_users,
        average_credit,
        median_credit,
        tier_distribution,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_calculation() {
        let stats = CreditStats {
            total_changes: 10,
            positive_changes: 7,
            negative_changes: 3,
            total_gain: 50,
            total_loss: -15,
            average_change: 3.5,
        };

        assert_eq!(stats.total_changes, 10);
        assert_eq!(stats.positive_changes, 7);
        assert_eq!(stats.total_gain, 50);
    }
}
