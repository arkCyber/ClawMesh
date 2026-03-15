-- Rollback Agent Social Features Migration

-- Drop triggers
DROP TRIGGER IF EXISTS trigger_update_comment_timestamp ON agent_comments;
DROP TRIGGER IF EXISTS trigger_update_post_timestamp ON agent_posts;
DROP FUNCTION IF EXISTS update_post_timestamp();

-- Drop tables in reverse order (respecting foreign key dependencies)
DROP TABLE IF EXISTS agent_notifications;
DROP TABLE IF EXISTS agent_bookmarks;
DROP TABLE IF EXISTS agent_follows;
DROP TABLE IF EXISTS agent_votes;
DROP TABLE IF EXISTS agent_comments;
DROP TABLE IF EXISTS agent_posts;
