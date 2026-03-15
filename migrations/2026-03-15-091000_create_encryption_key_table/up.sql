-- Encryption Key Persistence Table
-- Aerospace-grade database schema for storing encryption keys

-- ============================================================================
-- Encryption Key Table
-- ============================================================================
CREATE TABLE encryption_key (
    id VARCHAR(36) PRIMARY KEY,
    user_id INT NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    key_data BYTEA NOT NULL,
    algorithm VARCHAR(50) NOT NULL DEFAULT 'AES-256-GCM',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP,
    revoked_at TIMESTAMP,
    last_used_at TIMESTAMP,
    usage_count BIGINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    
    -- Constraints
    CONSTRAINT encryption_key_algorithm_check CHECK (
        algorithm IN ('AES-256-GCM', 'ChaCha20-Poly1305', 'AES-128-GCM')
    )
);

-- Indexes for efficient queries
CREATE INDEX idx_encryption_key_user_id ON encryption_key(user_id);
CREATE INDEX idx_encryption_key_created_at ON encryption_key(created_at DESC);
CREATE INDEX idx_encryption_key_is_active ON encryption_key(is_active) WHERE is_active = true;
CREATE INDEX idx_encryption_key_user_active ON encryption_key(user_id, is_active) WHERE is_active = true;

-- Composite index for active key lookup
CREATE INDEX idx_encryption_key_user_active_created ON encryption_key(user_id, created_at DESC) 
    WHERE is_active = true AND revoked_at IS NULL;

-- ============================================================================
-- Key Rotation History Table
-- ============================================================================
CREATE TABLE key_rotation_history (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    old_key_id VARCHAR(36) NOT NULL,
    new_key_id VARCHAR(36) NOT NULL,
    rotation_reason VARCHAR(100),
    rotated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    FOREIGN KEY (old_key_id) REFERENCES encryption_key(id) ON DELETE CASCADE,
    FOREIGN KEY (new_key_id) REFERENCES encryption_key(id) ON DELETE CASCADE
);

-- Index for rotation history
CREATE INDEX idx_key_rotation_user_id ON key_rotation_history(user_id);
CREATE INDEX idx_key_rotation_rotated_at ON key_rotation_history(rotated_at DESC);

-- ============================================================================
-- Triggers for automatic updates
-- ============================================================================

-- Update last_used_at on key usage
CREATE OR REPLACE FUNCTION update_key_last_used()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_used_at = NOW();
    NEW.usage_count = NEW.usage_count + 1;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Note: This trigger would be called from application code when key is used
-- CREATE TRIGGER encryption_key_usage_update
--     BEFORE UPDATE OF usage_count ON encryption_key
--     FOR EACH ROW
--     EXECUTE FUNCTION update_key_last_used();

-- ============================================================================
-- Helper Functions
-- ============================================================================

-- Get active key for user
CREATE OR REPLACE FUNCTION get_active_key_for_user(user_id_param INT)
RETURNS VARCHAR(36) AS $$
DECLARE
    key_id VARCHAR(36);
BEGIN
    SELECT id INTO key_id
    FROM encryption_key
    WHERE user_id = user_id_param
      AND is_active = true
      AND revoked_at IS NULL
      AND (expires_at IS NULL OR expires_at > NOW())
    ORDER BY created_at DESC
    LIMIT 1;
    
    RETURN key_id;
END;
$$ LANGUAGE plpgsql;

-- Revoke all keys for user
CREATE OR REPLACE FUNCTION revoke_all_user_keys(user_id_param INT)
RETURNS INT AS $$
DECLARE
    revoked_count INT;
BEGIN
    UPDATE encryption_key
    SET revoked_at = NOW(),
        is_active = false
    WHERE user_id = user_id_param
      AND revoked_at IS NULL;
    
    GET DIAGNOSTICS revoked_count = ROW_COUNT;
    RETURN revoked_count;
END;
$$ LANGUAGE plpgsql;

-- Clean up expired keys
CREATE OR REPLACE FUNCTION cleanup_expired_keys()
RETURNS INT AS $$
DECLARE
    deleted_count INT;
BEGIN
    -- Mark expired keys as inactive
    UPDATE encryption_key
    SET is_active = false
    WHERE expires_at IS NOT NULL
      AND expires_at < NOW()
      AND is_active = true;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Get key count for user
CREATE OR REPLACE FUNCTION get_user_key_count(user_id_param INT)
RETURNS INT AS $$
BEGIN
    RETURN (
        SELECT COUNT(*)
        FROM encryption_key
        WHERE user_id = user_id_param
          AND is_active = true
          AND revoked_at IS NULL
    );
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- Comments for documentation
-- ============================================================================

COMMENT ON TABLE encryption_key IS 'Stores encryption keys for end-to-end encryption';
COMMENT ON TABLE key_rotation_history IS 'Tracks key rotation events for audit purposes';

COMMENT ON COLUMN encryption_key.id IS 'Unique key identifier (UUID)';
COMMENT ON COLUMN encryption_key.user_id IS 'Owner of the encryption key';
COMMENT ON COLUMN encryption_key.key_data IS 'Encrypted key material (binary)';
COMMENT ON COLUMN encryption_key.algorithm IS 'Encryption algorithm used';
COMMENT ON COLUMN encryption_key.expires_at IS 'Optional key expiration timestamp';
COMMENT ON COLUMN encryption_key.revoked_at IS 'Timestamp when key was revoked';
COMMENT ON COLUMN encryption_key.last_used_at IS 'Last time key was used';
COMMENT ON COLUMN encryption_key.usage_count IS 'Number of times key has been used';
COMMENT ON COLUMN encryption_key.is_active IS 'Whether key is currently active';
