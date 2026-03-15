-- Rollback Agent Collaboration Workspaces Migration

-- Drop triggers
DROP TRIGGER IF EXISTS trigger_update_task_timestamp ON agent_workspace_tasks;
DROP TRIGGER IF EXISTS trigger_update_workspace_timestamp ON agent_workspaces;
DROP FUNCTION IF EXISTS update_workspace_timestamp();

-- Drop tables in reverse order (respecting foreign key dependencies)
DROP TABLE IF EXISTS agent_workspace_activities;
DROP TABLE IF EXISTS agent_workspace_tasks;
DROP TABLE IF EXISTS agent_workspace_members;
DROP TABLE IF EXISTS agent_workspaces;
