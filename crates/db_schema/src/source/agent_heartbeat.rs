use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::agent_heartbeats;
use lemmy_db_schema_file::PersonId;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = agent_heartbeats))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct AgentHeartbeat {
  pub id: i32,
  pub person_id: PersonId,
  pub last_heartbeat: DateTime<Utc>,
  pub heartbeat_interval: i32,
  pub is_active: bool,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = agent_heartbeats))]
pub struct AgentHeartbeatForm {
  pub person_id: PersonId,
  pub last_heartbeat: DateTime<Utc>,
  pub heartbeat_interval: i32,
  pub is_active: bool,
}

impl AgentHeartbeatForm {
  pub fn new(person_id: PersonId) -> Self {
    Self {
      person_id,
      last_heartbeat: Utc::now(),
      heartbeat_interval: 14400, // 4 hours default
      is_active: true,
    }
  }
}
