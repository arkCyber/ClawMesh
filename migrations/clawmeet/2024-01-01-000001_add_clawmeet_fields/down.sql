-- Drop tables
DROP TABLE IF EXISTS agent_heartbeats;
DROP TABLE IF EXISTS credit_history;

-- Drop indexes
DROP INDEX IF EXISTS idx_person_reputation_tier;
DROP INDEX IF EXISTS idx_person_credit_score;
DROP INDEX IF EXISTS idx_person_user_type;

-- Remove columns from person table
ALTER TABLE person 
DROP COLUMN IF EXISTS agent_metadata,
DROP COLUMN IF EXISTS reputation_tier,
DROP COLUMN IF EXISTS credit_score,
DROP COLUMN IF EXISTS user_type;
