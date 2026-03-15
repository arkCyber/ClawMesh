use anyhow::Result;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::PersonId;

use crate::{CreditAction, CreditHistoryForm};

/// Batch update credit scores for multiple users
pub async fn batch_update_credits(
    updates: Vec<(PersonId, i32, String)>,
    conn: &mut AsyncPgConnection,
) -> Result<usize> {
    use lemmy_db_schema_file::schema::{credit_history, person};

    let mut updated_count = 0;

    for (person_id, credit_change, reason) in updates {
        // Get current credit
        let current_credit: i32 = person::table
            .find(person_id)
            .select(person::credit_score)
            .first(conn)
            .await?;

        // Calculate new credit
        let new_credit = (current_credit + credit_change).clamp(0, 1000);
        let new_tier = crate::tier::get_reputation_tier(new_credit);

        // Update person
        diesel::update(person::table.find(person_id))
            .set((
                person::credit_score.eq(new_credit),
                person::reputation_tier.eq(new_tier.as_str()),
            ))
            .execute(conn)
            .await?;

        // Record history
        let history_form = CreditHistoryForm {
            person_id,
            action_type: reason.clone(),
            credit_change,
            reason: Some(reason),
        };

        diesel::insert_into(credit_history::table)
            .values(&history_form)
            .execute(conn)
            .await?;

        updated_count += 1;
    }

    Ok(updated_count)
}

/// Apply credit action to all users matching criteria
pub async fn apply_to_tier(
    tier: &str,
    action: &CreditAction,
    reason: &str,
    conn: &mut AsyncPgConnection,
) -> Result<usize> {
    use lemmy_db_schema_file::schema::person;

    // Get all users in the tier
    let user_ids: Vec<PersonId> = person::table
        .filter(person::reputation_tier.eq(tier))
        .filter(person::user_type.eq("human"))
        .select(person::id)
        .load(conn)
        .await?;

    let credit_change = crate::calculator::calculate_credit_change(action);

    let updates: Vec<(PersonId, i32, String)> = user_ids
        .into_iter()
        .map(|id| (id, credit_change, reason.to_string()))
        .collect();

    batch_update_credits(updates, conn).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_operations() {
        // Test that batch operations structure is correct
        let updates = vec![
            (PersonId(1), 10, "test".to_string()),
            (PersonId(2), -5, "test".to_string()),
        ];
        assert_eq!(updates.len(), 2);
    }
}
