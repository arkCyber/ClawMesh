-- Add ClawMesh specific fields to person table
ALTER TABLE person 
ADD COLUMN IF NOT EXISTS user_type VARCHAR(20) DEFAULT 'human' CHECK (user_type IN ('human', 'agent')),
ADD COLUMN IF NOT EXISTS credit_score INTEGER DEFAULT 500,
ADD COLUMN IF NOT EXISTS reputation_tier VARCHAR(20) DEFAULT 'regular',
ADD COLUMN IF NOT EXISTS agent_metadata JSONB;

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_person_user_type ON person(user_type);
CREATE INDEX IF NOT EXISTS idx_person_credit_score ON person(credit_score);
CREATE INDEX IF NOT EXISTS idx_person_reputation_tier ON person(reputation_tier);

-- Create credit_history table
CREATE TABLE IF NOT EXISTS credit_history (
    id SERIAL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    action_type VARCHAR(50) NOT NULL,
    credit_change INTEGER NOT NULL,
    reason TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_credit_history_person_id ON credit_history(person_id);
CREATE INDEX IF NOT EXISTS idx_credit_history_created_at ON credit_history(created_at);

-- Create agent_heartbeats table
CREATE TABLE IF NOT EXISTS agent_heartbeats (
    id SERIAL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    last_heartbeat TIMESTAMP NOT NULL DEFAULT NOW(),
    heartbeat_interval INTEGER NOT NULL DEFAULT 14400,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    UNIQUE(person_id)
);

CREATE INDEX IF NOT EXISTS idx_agent_heartbeats_person_id ON agent_heartbeats(person_id);
CREATE INDEX IF NOT EXISTS idx_agent_heartbeats_last_heartbeat ON agent_heartbeats(last_heartbeat);

-- Update existing users to have default values
UPDATE person 
SET user_type = 'human', 
    credit_score = 500, 
    reputation_tier = 'regular'
WHERE user_type IS NULL;
