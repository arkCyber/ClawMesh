pub mod batch;
pub mod calculator;
pub mod models;
pub mod permissions;
pub mod stats;
pub mod tier;
#[cfg(test)]
mod tests;

pub use batch::{apply_to_tier, batch_update_credits};
pub use calculator::{calculate_credit_change, CreditAction};
pub use models::{CreditHistory, CreditHistoryForm};
pub use permissions::{can_create_community, can_moderate, can_post, get_min_credit_for_action};
pub use stats::{get_global_stats, get_person_stats, CreditStats, GlobalStats};
pub use tier::{get_reputation_tier, ReputationTier};

use anyhow::Result;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::PersonId;

/// Update a person's credit score and reputation tier
pub async fn update_person_credit(
    person_id: PersonId,
    credit_change: i32,
    reason: &str,
    conn: &mut AsyncPgConnection,
) -> Result<i32> {
    use lemmy_db_schema_file::schema::{credit_history, person};

    // Get current credit score
    let current_credit: i32 = person::table
        .find(person_id)
        .select(person::credit_score)
        .first(conn)
        .await?;

    // Calculate new credit score (clamped between 0 and 1000)
    let new_credit = (current_credit + credit_change).clamp(0, 1000);
    let new_tier = get_reputation_tier(new_credit);

    // Update person's credit and tier
    diesel::update(person::table.find(person_id))
        .set((
            person::credit_score.eq(new_credit),
            person::reputation_tier.eq(new_tier.as_str()),
        ))
        .execute(conn)
        .await?;

    // Record credit history
    let history_form = CreditHistoryForm {
        person_id,
        action_type: reason.to_string(),
        credit_change,
        reason: Some(reason.to_string()),
    };

    diesel::insert_into(credit_history::table)
        .values(&history_form)
        .execute(conn)
        .await?;

    Ok(new_credit)
}

/// Get credit history for a person
pub async fn get_credit_history(
    person_id: PersonId,
    limit: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<CreditHistory>> {
    use lemmy_db_schema_file::schema::credit_history;

    let history = credit_history::table
        .filter(credit_history::person_id.eq(person_id))
        .order(credit_history::created_at.desc())
        .limit(limit)
        .load::<CreditHistory>(conn)
        .await?;

    Ok(history)
}
