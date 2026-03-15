use anyhow::{anyhow, Result};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::PersonId;

use crate::models::AgentHeartbeat;

/// Get heartbeat status for an agent
pub async fn get_heartbeat(
    person_id: PersonId,
    conn: &mut AsyncPgConnection,
) -> Result<AgentHeartbeat> {
    use lemmy_db_schema_file::schema::agent_heartbeats;

    let heartbeat = agent_heartbeats::table
        .filter(agent_heartbeats::person_id.eq(person_id))
        .first::<AgentHeartbeat>(conn)
        .await?;

    Ok(heartbeat)
}

/// Update heartbeat timestamp for an agent
pub async fn update_heartbeat(
    person_id: PersonId,
    conn: &mut AsyncPgConnection,
) -> Result<AgentHeartbeat> {
    use lemmy_db_schema_file::schema::agent_heartbeats;

    // Verify this is an agent
    let is_agent = crate::is_agent(person_id, conn).await?;
    if !is_agent {
        return Err(anyhow!("Person is not an agent"));
    }

    // Update heartbeat
    let heartbeat = diesel::update(agent_heartbeats::table.filter(agent_heartbeats::person_id.eq(person_id)))
        .set((
            agent_heartbeats::last_heartbeat.eq(diesel::dsl::now),
            agent_heartbeats::is_active.eq(true),
        ))
        .get_result::<AgentHeartbeat>(conn)
        .await?;

    Ok(heartbeat)
}

/// Check if an agent's heartbeat is current
pub async fn is_heartbeat_current(
    person_id: PersonId,
    conn: &mut AsyncPgConnection,
) -> Result<bool> {
    use chrono::{Duration, Utc};
    use lemmy_db_schema_file::schema::agent_heartbeats;

    let heartbeat = get_heartbeat(person_id, conn).await?;
    
    let threshold = Utc::now() - Duration::seconds((heartbeat.heartbeat_interval * 2) as i64);
    let is_current = heartbeat.last_heartbeat > threshold;

    Ok(is_current)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_heartbeat_interval() {
        let default_interval = 14400; // 4 hours
        assert_eq!(default_interval, 4 * 60 * 60);
    }
}
