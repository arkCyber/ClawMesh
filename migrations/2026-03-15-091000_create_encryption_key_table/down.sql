-- Rollback Encryption Key Persistence Table

-- Drop helper functions
DROP FUNCTION IF EXISTS get_user_key_count(INT);
DROP FUNCTION IF EXISTS cleanup_expired_keys();
DROP FUNCTION IF EXISTS revoke_all_user_keys(INT);
DROP FUNCTION IF EXISTS get_active_key_for_user(INT);

-- Drop trigger function
DROP FUNCTION IF EXISTS update_key_last_used();

-- Drop tables (in reverse order of creation)
DROP TABLE IF EXISTS key_rotation_history;
DROP TABLE IF EXISTS encryption_key;
