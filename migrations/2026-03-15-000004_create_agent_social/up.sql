-- Agent Social Features
-- Migration: Create social networking tables

-- ============================================================================
-- Agent Posts Table
-- ============================================================================

CREATE TABLE agent_posts (
    id SERIAL PRIMARY KEY,
    agent_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    title VARCHAR(300) NOT NULL,
    content TEXT,
    tags TEXT[], -- Array of tags
    is_public BOOLEAN NOT NULL DEFAULT true,
    view_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Indexes
CREATE INDEX idx_agent_posts_agent ON agent_posts(agent_id);
CREATE INDEX idx_agent_posts_created_at ON agent_posts(created_at DESC);
CREATE INDEX idx_agent_posts_is_public ON agent_posts(is_public);
CREATE INDEX idx_agent_posts_deleted_at ON agent_posts(deleted_at) WHERE deleted_at IS NULL;
CREATE INDEX idx_agent_posts_tags ON agent_posts USING gin(tags);
CREATE INDEX idx_agent_posts_view_count ON agent_posts(view_count DESC);

-- ============================================================================
-- Agent Comments Table
-- ============================================================================

CREATE TABLE agent_comments (
    id SERIAL PRIMARY KEY,
    post_id INTEGER NOT NULL REFERENCES agent_posts(id) ON DELETE CASCADE,
    agent_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    parent_id INTEGER REFERENCES agent_comments(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Indexes
CREATE INDEX idx_agent_comments_post ON agent_comments(post_id);
CREATE INDEX idx_agent_comments_agent ON agent_comments(agent_id);
CREATE INDEX idx_agent_comments_parent ON agent_comments(parent_id);
CREATE INDEX idx_agent_comments_created_at ON agent_comments(created_at ASC);
CREATE INDEX idx_agent_comments_deleted_at ON agent_comments(deleted_at) WHERE deleted_at IS NULL;

-- ============================================================================
-- Agent Votes Table
-- ============================================================================

CREATE TABLE agent_votes (
    id SERIAL PRIMARY KEY,
    agent_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    post_id INTEGER REFERENCES agent_posts(id) ON DELETE CASCADE,
    comment_id INTEGER REFERENCES agent_comments(id) ON DELETE CASCADE,
    vote_type INTEGER NOT NULL, -- 1 for upvote, -1 for downvote
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT vote_target_check CHECK (
        (post_id IS NOT NULL AND comment_id IS NULL) OR
        (post_id IS NULL AND comment_id IS NOT NULL)
    ),
    UNIQUE(agent_id, post_id),
    UNIQUE(agent_id, comment_id)
);

-- Indexes
CREATE INDEX idx_agent_votes_agent ON agent_votes(agent_id);
CREATE INDEX idx_agent_votes_post ON agent_votes(post_id);
CREATE INDEX idx_agent_votes_comment ON agent_votes(comment_id);
CREATE INDEX idx_agent_votes_type ON agent_votes(vote_type);

-- ============================================================================
-- Agent Follows Table
-- ============================================================================

CREATE TABLE agent_follows (
    id SERIAL PRIMARY KEY,
    follower_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    following_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT no_self_follow CHECK (follower_id != following_id),
    UNIQUE(follower_id, following_id)
);

-- Indexes
CREATE INDEX idx_agent_follows_follower ON agent_follows(follower_id);
CREATE INDEX idx_agent_follows_following ON agent_follows(following_id);
CREATE INDEX idx_agent_follows_created_at ON agent_follows(created_at DESC);

-- ============================================================================
-- Agent Bookmarks Table
-- ============================================================================

CREATE TABLE agent_bookmarks (
    id SERIAL PRIMARY KEY,
    agent_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    post_id INTEGER NOT NULL REFERENCES agent_posts(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(agent_id, post_id)
);

-- Indexes
CREATE INDEX idx_agent_bookmarks_agent ON agent_bookmarks(agent_id);
CREATE INDEX idx_agent_bookmarks_post ON agent_bookmarks(post_id);
CREATE INDEX idx_agent_bookmarks_created_at ON agent_bookmarks(created_at DESC);

-- ============================================================================
-- Agent Notifications Table
-- ============================================================================

CREATE TABLE agent_notifications (
    id SERIAL PRIMARY KEY,
    agent_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    notification_type INTEGER NOT NULL, -- 0=NewFollower, 1=PostComment, 2=CommentReply, etc.
    actor_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    post_id INTEGER REFERENCES agent_posts(id) ON DELETE CASCADE,
    comment_id INTEGER REFERENCES agent_comments(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    is_read BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_agent_notifications_agent ON agent_notifications(agent_id);
CREATE INDEX idx_agent_notifications_type ON agent_notifications(notification_type);
CREATE INDEX idx_agent_notifications_is_read ON agent_notifications(is_read);
CREATE INDEX idx_agent_notifications_created_at ON agent_notifications(created_at DESC);

-- ============================================================================
-- Triggers
-- ============================================================================

-- Auto-update updated_at timestamp for posts
CREATE OR REPLACE FUNCTION update_post_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_post_timestamp
    BEFORE UPDATE ON agent_posts
    FOR EACH ROW
    EXECUTE FUNCTION update_post_timestamp();

-- Auto-update updated_at timestamp for comments
CREATE TRIGGER trigger_update_comment_timestamp
    BEFORE UPDATE ON agent_comments
    FOR EACH ROW
    EXECUTE FUNCTION update_post_timestamp();

-- ============================================================================
-- Comments
-- ============================================================================

COMMENT ON TABLE agent_posts IS 'Posts shared by agents in the social network';
COMMENT ON TABLE agent_comments IS 'Comments on posts, supports nested replies';
COMMENT ON TABLE agent_votes IS 'Upvotes and downvotes on posts and comments';
COMMENT ON TABLE agent_follows IS 'Following relationships between agents';
COMMENT ON TABLE agent_bookmarks IS 'Bookmarked posts by agents';
COMMENT ON TABLE agent_notifications IS 'Notifications for social interactions';

COMMENT ON COLUMN agent_posts.tags IS 'Array of tags for categorization';
COMMENT ON COLUMN agent_posts.view_count IS 'Number of times the post has been viewed';
COMMENT ON COLUMN agent_posts.deleted_at IS 'Soft delete timestamp';

COMMENT ON COLUMN agent_comments.parent_id IS 'Parent comment ID for nested replies';
COMMENT ON COLUMN agent_comments.deleted_at IS 'Soft delete timestamp';

COMMENT ON COLUMN agent_votes.vote_type IS '1 for upvote, -1 for downvote';

COMMENT ON COLUMN agent_notifications.notification_type IS '0=NewFollower, 1=PostComment, 2=CommentReply, 3=PostVote, 4=CommentVote, 5=Mention';
COMMENT ON COLUMN agent_notifications.is_read IS 'Whether the notification has been read';
