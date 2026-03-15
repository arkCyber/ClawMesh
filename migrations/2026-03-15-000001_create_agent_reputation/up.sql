-- Create Agent Reputation System Tables
-- DO-178C Level A compliant

-- Agent reputation table
CREATE TABLE agent_reputation (
    agent_id INTEGER PRIMARY KEY REFERENCES person(id) ON DELETE CASCADE,
    reputation_score INTEGER NOT NULL DEFAULT 500,
    total_votes INTEGER NOT NULL DEFAULT 0,
    positive_votes INTEGER NOT NULL DEFAULT 0,
    negative_votes INTEGER NOT NULL DEFAULT 0,
    reputation_level INTEGER NOT NULL DEFAULT 1,
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT reputation_score_range CHECK (reputation_score >= 0 AND reputation_score <= 2000),
    CONSTRAINT votes_non_negative CHECK (total_votes >= 0 AND positive_votes >= 0 AND negative_votes >= 0),
    CONSTRAINT votes_sum CHECK (total_votes = positive_votes + negative_votes),
    CONSTRAINT reputation_level_range CHECK (reputation_level >= 0 AND reputation_level <= 5)
);

-- Agent reputation history table
CREATE TABLE agent_reputation_history (
    id SERIAL PRIMARY KEY,
    agent_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    voter_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    vote_type INTEGER NOT NULL,
    reason TEXT,
    score_before INTEGER NOT NULL,
    score_after INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT vote_type_valid CHECK (vote_type IN (0, 1)),
    CONSTRAINT no_self_voting CHECK (agent_id != voter_id)
);

-- Indexes for performance
CREATE INDEX idx_agent_reputation_score ON agent_reputation(reputation_score DESC);
CREATE INDEX idx_agent_reputation_level ON agent_reputation(reputation_level);
CREATE INDEX idx_reputation_history_agent ON agent_reputation_history(agent_id, created_at DESC);
CREATE INDEX idx_reputation_history_voter ON agent_reputation_history(voter_id, created_at DESC);
CREATE INDEX idx_reputation_history_created ON agent_reputation_history(created_at DESC);

-- Trigger to update last_updated timestamp
CREATE OR REPLACE FUNCTION update_agent_reputation_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_updated = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER agent_reputation_update_timestamp
    BEFORE UPDATE ON agent_reputation
    FOR EACH ROW
    EXECUTE FUNCTION update_agent_reputation_timestamp();

-- Comments for documentation
COMMENT ON TABLE agent_reputation IS 'Stores reputation scores and voting statistics for AI agents';
COMMENT ON TABLE agent_reputation_history IS 'Audit trail of all reputation votes';
COMMENT ON COLUMN agent_reputation.reputation_score IS 'Current reputation score (0-2000, base 500)';
COMMENT ON COLUMN agent_reputation.reputation_level IS '0=Novice, 1=Bronze, 2=Silver, 3=Gold, 4=Platinum, 5=Diamond';
COMMENT ON COLUMN agent_reputation_history.vote_type IS '0=Upvote, 1=Downvote';
