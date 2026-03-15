-- ClawMesh Friendship System Tables
-- Aerospace-grade database schema with full integrity constraints

-- ============================================================================
-- Friendship Table
-- ============================================================================
CREATE TABLE friendship (
    id SERIAL PRIMARY KEY,
    user_id_1 INT NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    user_id_2 INT NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    -- Ensure user_id_1 < user_id_2 to avoid duplicates
    CONSTRAINT friendship_user_order CHECK (user_id_1 < user_id_2),
    CONSTRAINT friendship_unique UNIQUE(user_id_1, user_id_2)
);

-- Indexes for efficient queries
CREATE INDEX idx_friendship_user_id_1 ON friendship(user_id_1);
CREATE INDEX idx_friendship_user_id_2 ON friendship(user_id_2);
CREATE INDEX idx_friendship_created_at ON friendship(created_at DESC);

-- ============================================================================
-- Friend Request Table
-- ============================================================================
CREATE TABLE friend_request (
    id SERIAL PRIMARY KEY,
    sender_id INT NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    recipient_id INT NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    message TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    responded_at TIMESTAMP,
    
    -- Status must be one of: pending, accepted, rejected, cancelled
    CONSTRAINT friend_request_status_check CHECK (
        status IN ('pending', 'accepted', 'rejected', 'cancelled')
    ),
    
    -- Cannot send request to yourself
    CONSTRAINT friend_request_not_self CHECK (sender_id != recipient_id),
    
    -- Only one pending request between two users
    CONSTRAINT friend_request_unique UNIQUE(sender_id, recipient_id)
);

-- Indexes for efficient queries
CREATE INDEX idx_friend_request_sender_id ON friend_request(sender_id);
CREATE INDEX idx_friend_request_recipient_id ON friend_request(recipient_id);
CREATE INDEX idx_friend_request_status ON friend_request(status);
CREATE INDEX idx_friend_request_created_at ON friend_request(created_at DESC);

-- Composite index for pending requests
CREATE INDEX idx_friend_request_recipient_pending ON friend_request(recipient_id, status) 
    WHERE status = 'pending';

-- ============================================================================
-- User Block Table
-- ============================================================================
CREATE TABLE user_block (
    id SERIAL PRIMARY KEY,
    blocker_id INT NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    blocked_id INT NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    reason TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    -- Cannot block yourself
    CONSTRAINT user_block_not_self CHECK (blocker_id != blocked_id),
    
    -- Only one block entry per pair
    CONSTRAINT user_block_unique UNIQUE(blocker_id, blocked_id)
);

-- Indexes for efficient queries
CREATE INDEX idx_user_block_blocker_id ON user_block(blocker_id);
CREATE INDEX idx_user_block_blocked_id ON user_block(blocked_id);
CREATE INDEX idx_user_block_created_at ON user_block(created_at DESC);

-- ============================================================================
-- Friend Nickname Table (Optional customization)
-- ============================================================================
CREATE TABLE friend_nickname (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    friend_id INT NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    nickname VARCHAR(100) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    -- Cannot set nickname for yourself
    CONSTRAINT friend_nickname_not_self CHECK (user_id != friend_id),
    
    -- Only one nickname per friend
    CONSTRAINT friend_nickname_unique UNIQUE(user_id, friend_id)
);

-- Indexes for efficient queries
CREATE INDEX idx_friend_nickname_user_id ON friend_nickname(user_id);
CREATE INDEX idx_friend_nickname_friend_id ON friend_nickname(friend_id);

-- ============================================================================
-- Triggers for automatic timestamp updates
-- ============================================================================

-- Update friendship.updated_at on change
CREATE OR REPLACE FUNCTION update_friendship_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER friendship_update_timestamp
    BEFORE UPDATE ON friendship
    FOR EACH ROW
    EXECUTE FUNCTION update_friendship_timestamp();

-- Update friend_nickname.updated_at on change
CREATE OR REPLACE FUNCTION update_friend_nickname_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER friend_nickname_update_timestamp
    BEFORE UPDATE ON friend_nickname
    FOR EACH ROW
    EXECUTE FUNCTION update_friend_nickname_timestamp();

-- ============================================================================
-- Helper Functions
-- ============================================================================

-- Function to check if two users are friends
CREATE OR REPLACE FUNCTION are_friends(user1_id INT, user2_id INT)
RETURNS BOOLEAN AS $$
DECLARE
    min_id INT;
    max_id INT;
BEGIN
    -- Normalize user IDs
    IF user1_id < user2_id THEN
        min_id := user1_id;
        max_id := user2_id;
    ELSE
        min_id := user2_id;
        max_id := user1_id;
    END IF;
    
    -- Check if friendship exists
    RETURN EXISTS (
        SELECT 1 FROM friendship
        WHERE user_id_1 = min_id AND user_id_2 = max_id
    );
END;
$$ LANGUAGE plpgsql;

-- Function to check if user is blocked
CREATE OR REPLACE FUNCTION is_blocked(blocker INT, blocked INT)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 FROM user_block
        WHERE blocker_id = blocker AND blocked_id = blocked
    );
END;
$$ LANGUAGE plpgsql;

-- Function to get friend count
CREATE OR REPLACE FUNCTION get_friend_count(user_id_param INT)
RETURNS INT AS $$
BEGIN
    RETURN (
        SELECT COUNT(*)
        FROM friendship
        WHERE user_id_1 = user_id_param OR user_id_2 = user_id_param
    );
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- Comments for documentation
-- ============================================================================

COMMENT ON TABLE friendship IS 'Stores confirmed friend relationships between users';
COMMENT ON TABLE friend_request IS 'Stores pending, accepted, rejected, and cancelled friend requests';
COMMENT ON TABLE user_block IS 'Stores user blocking relationships';
COMMENT ON TABLE friend_nickname IS 'Stores custom nicknames users assign to their friends';

COMMENT ON COLUMN friendship.user_id_1 IS 'Lower user ID (enforced by CHECK constraint)';
COMMENT ON COLUMN friendship.user_id_2 IS 'Higher user ID (enforced by CHECK constraint)';
COMMENT ON COLUMN friend_request.status IS 'Request status: pending, accepted, rejected, cancelled';
COMMENT ON COLUMN user_block.reason IS 'Optional reason for blocking (for user reference)';
