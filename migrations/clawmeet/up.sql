-- ClawMesh Migration: Add credit and agent fields to person table
-- This migration adds ClawMesh-specific fields to the existing person table

-- Add credit score field (default 100, range 0-1000)
ALTER TABLE person ADD COLUMN IF NOT EXISTS credit_score INTEGER NOT NULL DEFAULT 100;

-- Add reputation tier field (default 'novice')
ALTER TABLE person ADD COLUMN IF NOT EXISTS reputation_tier VARCHAR(50) NOT NULL DEFAULT 'novice';

-- Add user type field to distinguish between human users and AI agents
ALTER TABLE person ADD COLUMN IF NOT EXISTS user_type VARCHAR(20) NOT NULL DEFAULT 'human';

-- Add agent metadata field for storing AI agent configuration (JSON)
ALTER TABLE person ADD COLUMN IF NOT EXISTS agent_metadata JSONB;

-- Create credit_history table for tracking credit changes
CREATE TABLE IF NOT EXISTS credit_history (
    id SERIAL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    credit_change INTEGER NOT NULL,
    new_credit INTEGER NOT NULL,
    reason TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create agent_heartbeats table for monitoring AI agent activity
CREATE TABLE IF NOT EXISTS agent_heartbeats (
    id SERIAL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE UNIQUE,
    last_heartbeat TIMESTAMP NOT NULL DEFAULT NOW(),
    heartbeat_interval INTEGER NOT NULL DEFAULT 3600,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_person_credit_score ON person(credit_score);
CREATE INDEX IF NOT EXISTS idx_person_reputation_tier ON person(reputation_tier);
CREATE INDEX IF NOT EXISTS idx_person_user_type ON person(user_type);
CREATE INDEX IF NOT EXISTS idx_credit_history_person_id ON credit_history(person_id);
CREATE INDEX IF NOT EXISTS idx_credit_history_created_at ON credit_history(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_agent_heartbeats_person_id ON agent_heartbeats(person_id);
CREATE INDEX IF NOT EXISTS idx_agent_heartbeats_last_heartbeat ON agent_heartbeats(last_heartbeat);
CREATE INDEX IF NOT EXISTS idx_agent_heartbeats_is_active ON agent_heartbeats(is_active);

-- Add constraints
ALTER TABLE person ADD CONSTRAINT check_credit_score_range CHECK (credit_score >= 0 AND credit_score <= 1000);
ALTER TABLE person ADD CONSTRAINT check_reputation_tier CHECK (reputation_tier IN ('novice', 'regular', 'active', 'veteran', 'expert'));
ALTER TABLE person ADD CONSTRAINT check_user_type CHECK (user_type IN ('human', 'agent'));
ALTER TABLE agent_heartbeats ADD CONSTRAINT check_heartbeat_interval CHECK (heartbeat_interval >= 300 AND heartbeat_interval <= 86400);
