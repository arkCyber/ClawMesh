use anyhow::Result;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::PersonId;

use crate::tier::ReputationTier;

/// Check if a person has permission to perform moderation actions
pub async fn can_moderate(person_id: PersonId, conn: &mut AsyncPgConnection) -> Result<bool> {
    use lemmy_db_schema_file::schema::person;

    let credit_score: i32 = person::table
        .find(person_id)
        .select(person::credit_score)
        .first(conn)
        .await?;

    let tier = crate::tier::get_reputation_tier(credit_score);
    
    // Only Trusted, Veteran, and Expert can moderate
    Ok(matches!(tier, ReputationTier::Active | ReputationTier::Veteran | ReputationTier::Expert))
}

/// Check if a person has permission to create communities
pub async fn can_create_community(person_id: PersonId, conn: &mut AsyncPgConnection) -> Result<bool> {
    use lemmy_db_schema_file::schema::person;

    let credit_score: i32 = person::table
        .find(person_id)
        .select(person::credit_score)
        .first(conn)
        .await?;

    // Need at least Regular tier (201+) to create communities
    Ok(credit_score >= 201)
}

/// Check if a person has permission to post
pub async fn can_post(person_id: PersonId, conn: &mut AsyncPgConnection) -> Result<bool> {
    use lemmy_db_schema_file::schema::person;

    let credit_score: i32 = person::table
        .find(person_id)
        .select(person::credit_score)
        .first(conn)
        .await?;

    // Need at least 50 credit to post (prevents spam from new accounts)
    Ok(credit_score >= 50)
}

/// Get minimum credit required for an action
pub fn get_min_credit_for_action(action: &str) -> i32 {
    match action {
        "post" => 50,
        "comment" => 0,
        "create_community" => 201,
        "moderate" => 501,
        "admin" => 701,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_credit_requirements() {
        assert_eq!(get_min_credit_for_action("post"), 50);
        assert_eq!(get_min_credit_for_action("comment"), 0);
        assert_eq!(get_min_credit_for_action("create_community"), 201);
        assert_eq!(get_min_credit_for_action("moderate"), 501);
        assert_eq!(get_min_credit_for_action("admin"), 701);
        assert_eq!(get_min_credit_for_action("unknown"), 0);
    }
}
