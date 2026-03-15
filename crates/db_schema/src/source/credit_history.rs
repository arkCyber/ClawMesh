use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::credit_history;
use lemmy_db_schema_file::PersonId;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = credit_history))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct CreditHistory {
  pub id: i32,
  pub person_id: PersonId,
  pub action_type: String,
  pub credit_change: i32,
  pub reason: Option<String>,
  pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = credit_history))]
pub struct CreditHistoryForm {
  pub person_id: PersonId,
  pub action_type: String,
  pub credit_change: i32,
  pub reason: Option<String>,
}

impl CreditHistoryForm {
  pub fn new(person_id: PersonId, action_type: String, credit_change: i32, reason: Option<String>) -> Self {
    Self {
      person_id,
      action_type,
      credit_change,
      reason,
    }
  }
}
