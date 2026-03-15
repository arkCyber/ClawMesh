pub mod heartbeat;
pub mod install;
pub mod list;
pub mod models;
pub mod validation;
#[cfg(test)]
mod tests;

pub use heartbeat::{get_heartbeat, is_heartbeat_current, update_heartbeat};
pub use install::install_agent;
pub use list::{count_agents, get_agent_info, get_stale_agents, list_agents, AgentInfo};
pub use models::{AgentHeartbeat, AgentHeartbeatForm};
pub use validation::{validate_heartbeat_interval, validate_metadata, validate_username};

use anyhow::Result;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::PersonId;

/// Check if a person is an agent
pub async fn is_agent(person_id: PersonId, conn: &mut AsyncPgConnection) -> Result<bool> {
    use lemmy_db_schema_file::schema::person;

    let user_type: String = person::table
        .find(person_id)
        .select(person::user_type)
        .first(conn)
        .await?;

    Ok(user_type == "agent")
}

/// Get all active agents
pub async fn get_active_agents(conn: &mut AsyncPgConnection) -> Result<Vec<PersonId>> {
    use lemmy_db_schema_file::schema::{agent_heartbeats, person};

    let agent_ids = person::table
        .inner_join(agent_heartbeats::table.on(person::id.eq(agent_heartbeats::person_id)))
        .filter(person::user_type.eq("agent"))
        .filter(agent_heartbeats::is_active.eq(true))
        .select(person::id)
        .load::<PersonId>(conn)
        .await?;

    Ok(agent_ids)
}

/// Mark inactive agents (no heartbeat in 2x interval)
pub async fn mark_inactive_agents(conn: &mut AsyncPgConnection) -> Result<usize> {
    use lemmy_db_schema_file::schema::agent_heartbeats;

    let count = diesel::update(agent_heartbeats::table)
        .filter(
            agent_heartbeats::is_active.eq(true)
        )
        .set(agent_heartbeats::is_active.eq(false))
        .execute(conn)
        .await?;

    Ok(count)
}
