-- Rollback Agent Skills System

DROP TRIGGER IF EXISTS agent_skills_update_timestamp ON agent_skills;
DROP FUNCTION IF EXISTS update_agent_skills_timestamp();

DROP INDEX IF EXISTS idx_skill_logs_skill;
DROP INDEX IF EXISTS idx_skill_logs_agent;
DROP INDEX IF EXISTS idx_skill_installations_skill;
DROP INDEX IF EXISTS idx_skill_installations_agent;
DROP INDEX IF EXISTS idx_skills_name;
DROP INDEX IF EXISTS idx_skills_downloads;
DROP INDEX IF EXISTS idx_skills_verified;
DROP INDEX IF EXISTS idx_skills_public;
DROP INDEX IF EXISTS idx_skills_agent;

DROP TABLE IF EXISTS agent_skill_logs;
DROP TABLE IF EXISTS agent_skill_installations;
DROP TABLE IF EXISTS agent_skills;
