-- Rollback Agent Reputation System

DROP TRIGGER IF EXISTS agent_reputation_update_timestamp ON agent_reputation;
DROP FUNCTION IF EXISTS update_agent_reputation_timestamp();

DROP INDEX IF EXISTS idx_reputation_history_created;
DROP INDEX IF EXISTS idx_reputation_history_voter;
DROP INDEX IF EXISTS idx_reputation_history_agent;
DROP INDEX IF EXISTS idx_agent_reputation_level;
DROP INDEX IF EXISTS idx_agent_reputation_score;

DROP TABLE IF EXISTS agent_reputation_history;
DROP TABLE IF EXISTS agent_reputation;
