-- Create chat groups table
CREATE TABLE chat_groups (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    group_type VARCHAR(50) NOT NULL CHECK (group_type IN ('private', 'public', 'secret')),
    creator_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    member_count INTEGER NOT NULL DEFAULT 0,
    max_members INTEGER,
    avatar_url TEXT,
    is_archived BOOLEAN NOT NULL DEFAULT FALSE,
    CONSTRAINT valid_member_count CHECK (member_count >= 0),
    CONSTRAINT valid_max_members CHECK (max_members IS NULL OR max_members > 0)
);

-- Create channels table
CREATE TABLE channels (
    id SERIAL PRIMARY KEY,
    group_id INTEGER NOT NULL REFERENCES chat_groups(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    channel_type VARCHAR(50) NOT NULL CHECK (channel_type IN ('text', 'voice', 'announcement')),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    is_archived BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    UNIQUE(group_id, name)
);

-- Create group members table
CREATE TABLE group_members (
    id SERIAL PRIMARY KEY,
    group_id INTEGER NOT NULL REFERENCES chat_groups(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL CHECK (role IN ('owner', 'admin', 'moderator', 'member', 'guest')),
    joined_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_active_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    is_muted BOOLEAN NOT NULL DEFAULT FALSE,
    is_banned BOOLEAN NOT NULL DEFAULT FALSE,
    nickname VARCHAR(255),
    UNIQUE(group_id, user_id)
);

-- Create group messages table
CREATE TABLE group_messages (
    id SERIAL PRIMARY KEY,
    channel_id INTEGER NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    sender_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    priority VARCHAR(50) NOT NULL DEFAULT 'normal' CHECK (priority IN ('low', 'normal', 'high', 'urgent')),
    status VARCHAR(50) NOT NULL DEFAULT 'sent' CHECK (status IN ('sent', 'delivered', 'read', 'failed')),
    reply_to_id INTEGER REFERENCES group_messages(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    edited_at TIMESTAMP WITH TIME ZONE,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    attachments TEXT[] DEFAULT ARRAY[]::TEXT[]
);

-- Create indexes for performance
CREATE INDEX idx_chat_groups_creator ON chat_groups(creator_id);
CREATE INDEX idx_chat_groups_type ON chat_groups(group_type);
CREATE INDEX idx_chat_groups_archived ON chat_groups(is_archived);

CREATE INDEX idx_channels_group ON channels(group_id);
CREATE INDEX idx_channels_type ON channels(channel_type);
CREATE INDEX idx_channels_archived ON channels(is_archived);

CREATE INDEX idx_group_members_group ON group_members(group_id);
CREATE INDEX idx_group_members_user ON group_members(user_id);
CREATE INDEX idx_group_members_role ON group_members(role);

CREATE INDEX idx_group_messages_channel ON group_messages(channel_id);
CREATE INDEX idx_group_messages_sender ON group_messages(sender_id);
CREATE INDEX idx_group_messages_created ON group_messages(created_at DESC);
CREATE INDEX idx_group_messages_reply ON group_messages(reply_to_id);

-- Create full-text search index for messages
CREATE INDEX idx_group_messages_content_fts ON group_messages USING gin(to_tsvector('english', content));

-- Create trigger to update member count
CREATE OR REPLACE FUNCTION update_group_member_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE chat_groups SET member_count = member_count + 1 WHERE id = NEW.group_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE chat_groups SET member_count = member_count - 1 WHERE id = OLD.group_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_member_count
AFTER INSERT OR DELETE ON group_members
FOR EACH ROW EXECUTE FUNCTION update_group_member_count();

-- Create trigger to update updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_chat_groups_updated_at
BEFORE UPDATE ON chat_groups
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
