use anyhow::Result;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema::source::person::Person;
use lemmy_db_schema_file::PersonId;
use serde::{Deserialize, Serialize};

use crate::models::AgentHeartbeat;

/// Agent with heartbeat information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub person: Person,
    pub heartbeat: AgentHeartbeat,
}

/// List all agents with optional filters
pub async fn list_agents(
    active_only: bool,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentInfo>> {
    use lemmy_db_schema_file::schema::{agent_heartbeats, person};

    let mut query = person::table
        .inner_join(agent_heartbeats::table.on(person::id.eq(agent_heartbeats::person_id)))
        .filter(person::user_type.eq("agent"))
        .into_boxed();

    if active_only {
        query = query.filter(agent_heartbeats::is_active.eq(true));
    }

    let results: Vec<(Person, AgentHeartbeat)> = query
        .order(agent_heartbeats::last_heartbeat.desc())
        .limit(limit)
        .offset(offset)
        .load(conn)
        .await?;

    let agent_infos = results
        .into_iter()
        .map(|(person, heartbeat)| AgentInfo { person, heartbeat })
        .collect();

    Ok(agent_infos)
}

/// Get agent by ID with heartbeat info
pub async fn get_agent_info(
    person_id: PersonId,
    conn: &mut AsyncPgConnection,
) -> Result<AgentInfo> {
    use lemmy_db_schema_file::schema::{agent_heartbeats, person};

    let (person, heartbeat): (Person, AgentHeartbeat) = person::table
        .inner_join(agent_heartbeats::table.on(person::id.eq(agent_heartbeats::person_id)))
        .filter(person::id.eq(person_id))
        .filter(person::user_type.eq("agent"))
        .first(conn)
        .await?;

    Ok(AgentInfo { person, heartbeat })
}

/// Count total agents
pub async fn count_agents(active_only: bool, conn: &mut AsyncPgConnection) -> Result<i64> {
    use lemmy_db_schema_file::schema::{agent_heartbeats, person};

    let mut query = person::table
        .inner_join(agent_heartbeats::table.on(person::id.eq(agent_heartbeats::person_id)))
        .filter(person::user_type.eq("agent"))
        .into_boxed();

    if active_only {
        query = query.filter(agent_heartbeats::is_active.eq(true));
    }

    let count: i64 = query.count().get_result(conn).await?;

    Ok(count)
}

/// Get agents that need heartbeat (haven't sent in a while)
pub async fn get_stale_agents(
    hours: i32,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<AgentInfo>> {
    use chrono::Utc;
    use lemmy_db_schema_file::schema::{agent_heartbeats, person};

    let threshold = Utc::now() - chrono::Duration::hours(hours as i64);

    let results: Vec<(Person, AgentHeartbeat)> = person::table
        .inner_join(agent_heartbeats::table.on(person::id.eq(agent_heartbeats::person_id)))
        .filter(person::user_type.eq("agent"))
        .filter(agent_heartbeats::last_heartbeat.lt(threshold))
        .filter(agent_heartbeats::is_active.eq(true))
        .load(conn)
        .await?;

    let agent_infos = results
        .into_iter()
        .map(|(person, heartbeat)| AgentInfo { person, heartbeat })
        .collect();

    Ok(agent_infos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_info_structure() {
        // Test that the structure is correctly defined
        assert_eq!(std::mem::size_of::<AgentInfo>(), std::mem::size_of::<(Person, AgentHeartbeat)>());
    }
}
