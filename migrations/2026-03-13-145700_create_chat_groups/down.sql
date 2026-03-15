-- Drop triggers
DROP TRIGGER IF EXISTS trigger_chat_groups_updated_at ON chat_groups;
DROP TRIGGER IF EXISTS trigger_update_member_count ON group_members;

-- Drop functions
DROP FUNCTION IF EXISTS update_updated_at_column();
DROP FUNCTION IF EXISTS update_group_member_count();

-- Drop indexes
DROP INDEX IF EXISTS idx_group_messages_content_fts;
DROP INDEX IF EXISTS idx_group_messages_reply;
DROP INDEX IF EXISTS idx_group_messages_created;
DROP INDEX IF EXISTS idx_group_messages_sender;
DROP INDEX IF EXISTS idx_group_messages_channel;

DROP INDEX IF EXISTS idx_group_members_role;
DROP INDEX IF EXISTS idx_group_members_user;
DROP INDEX IF EXISTS idx_group_members_group;

DROP INDEX IF EXISTS idx_channels_archived;
DROP INDEX IF EXISTS idx_channels_type;
DROP INDEX IF EXISTS idx_channels_group;

DROP INDEX IF EXISTS idx_chat_groups_archived;
DROP INDEX IF EXISTS idx_chat_groups_type;
DROP INDEX IF EXISTS idx_chat_groups_creator;

-- Drop tables
DROP TABLE IF EXISTS group_messages;
DROP TABLE IF EXISTS group_members;
DROP TABLE IF EXISTS channels;
DROP TABLE IF EXISTS chat_groups;
