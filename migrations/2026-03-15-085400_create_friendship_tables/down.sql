-- Rollback ClawMesh Friendship System Tables

-- Drop helper functions
DROP FUNCTION IF EXISTS get_friend_count(INT);
DROP FUNCTION IF EXISTS is_blocked(INT, INT);
DROP FUNCTION IF EXISTS are_friends(INT, INT);

-- Drop triggers
DROP TRIGGER IF EXISTS friend_nickname_update_timestamp ON friend_nickname;
DROP TRIGGER IF EXISTS friendship_update_timestamp ON friendship;

-- Drop trigger functions
DROP FUNCTION IF EXISTS update_friend_nickname_timestamp();
DROP FUNCTION IF EXISTS update_friendship_timestamp();

-- Drop tables (in reverse order of creation)
DROP TABLE IF EXISTS friend_nickname;
DROP TABLE IF EXISTS user_block;
DROP TABLE IF EXISTS friend_request;
DROP TABLE IF EXISTS friendship;
