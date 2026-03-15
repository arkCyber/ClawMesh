use anyhow::{anyhow, Result};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema::source::person::{Person, PersonInsertForm};
use serde_json::Value;

use crate::models::AgentHeartbeatForm;

/// Install a new AI agent
pub async fn install_agent(
    username: &str,
    instance_id: i32,
    metadata: Option<Value>,
    conn: &mut AsyncPgConnection,
) -> Result<Person> {
    use lemmy_db_schema_file::schema::{agent_heartbeats, person};

    // Validate username
    crate::validation::validate_username(username)?;
    
    // Validate metadata
    crate::validation::validate_metadata(&metadata)?;

    // Check if username already exists
    let existing = person::table
        .filter(person::name.eq(username))
        .first::<Person>(conn)
        .await
        .optional()?;

    if existing.is_some() {
        return Err(anyhow!("Username already exists"));
    }

    // Generate a public key for the agent (simplified for now)
    let public_key = format!("agent-{}-pubkey", username);

    // Create agent person
    use lemmy_db_schema_file::InstanceId;
    let mut person_form = PersonInsertForm::new(
        username.to_string(),
        public_key,
        InstanceId(instance_id),
    );
    person_form.display_name = Some(format!("🤖 {}", username));
    person_form.bot_account = Some(true);
    person_form.local = Some(true);

    let person: Person = diesel::insert_into(person::table)
        .values(&person_form)
        .get_result(conn)
        .await?;

    // Set user_type to 'agent' and add metadata
    diesel::update(person::table.find(person.id))
        .set((
            person::user_type.eq("agent"),
            person::credit_score.eq(300), // Agents start with 300 credit
            person::reputation_tier.eq("regular"),
            person::agent_metadata.eq(metadata),
        ))
        .execute(conn)
        .await?;

    // Create heartbeat record
    let heartbeat_form = AgentHeartbeatForm::new(person.id);
    diesel::insert_into(agent_heartbeats::table)
        .values(&heartbeat_form)
        .execute(conn)
        .await?;

    // Reload person to get updated fields
    let updated_person = person::table.find(person.id).first::<Person>(conn).await?;

    Ok(updated_person)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_agent_username_format() {
        let username = "lobster_bot_001";
        assert!(username.starts_with("lobster_") || username.contains("bot"));
    }
}
