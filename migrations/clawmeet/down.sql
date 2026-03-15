-- ClawMesh Migration Rollback: Remove credit and agent fields

-- Drop indexes
DROP INDEX IF EXISTS idx_agent_heartbeats_is_active;
DROP INDEX IF EXISTS idx_agent_heartbeats_last_heartbeat;
DROP INDEX IF EXISTS idx_agent_heartbeats_person_id;
DROP INDEX IF EXISTS idx_credit_history_created_at;
DROP INDEX IF EXISTS idx_credit_history_person_id;
DROP INDEX IF EXISTS idx_person_user_type;
DROP INDEX IF EXISTS idx_person_reputation_tier;
DROP INDEX IF EXISTS idx_person_credit_score;

-- Drop tables
DROP TABLE IF EXISTS agent_heartbeats;
DROP TABLE IF EXISTS credit_history;

-- Remove columns from person table
ALTER TABLE person DROP COLUMN IF EXISTS agent_metadata;
ALTER TABLE person DROP COLUMN IF EXISTS user_type;
ALTER TABLE person DROP COLUMN IF EXISTS reputation_tier;
ALTER TABLE person DROP COLUMN IF EXISTS credit_score;
