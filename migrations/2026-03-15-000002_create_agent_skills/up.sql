-- Create Agent Skills System Tables
-- DO-178C Level A compliant

-- Agent skills table
CREATE TABLE agent_skills (
    id SERIAL PRIMARY KEY,
    agent_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    skill_name VARCHAR(255) NOT NULL,
    skill_type INTEGER NOT NULL,
    skill_code TEXT,
    skill_metadata JSONB,
    version VARCHAR(50) NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    downloads INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT skill_type_valid CHECK (skill_type IN (0, 1, 2, 3)),
    CONSTRAINT downloads_non_negative CHECK (downloads >= 0),
    CONSTRAINT unique_skill_per_agent UNIQUE (agent_id, skill_name)
);

-- Agent skill installations table
CREATE TABLE agent_skill_installations (
    id SERIAL PRIMARY KEY,
    agent_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    skill_id INTEGER NOT NULL REFERENCES agent_skills(id) ON DELETE CASCADE,
    installed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_used TIMESTAMP WITH TIME ZONE,
    usage_count INTEGER NOT NULL DEFAULT 0,
    
    -- Constraints
    CONSTRAINT usage_count_non_negative CHECK (usage_count >= 0),
    CONSTRAINT unique_installation UNIQUE (agent_id, skill_id)
);

-- Agent skill execution logs table
CREATE TABLE agent_skill_logs (
    id SERIAL PRIMARY KEY,
    agent_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    skill_id INTEGER NOT NULL REFERENCES agent_skills(id) ON DELETE CASCADE,
    execution_time_ms INTEGER NOT NULL,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT execution_time_non_negative CHECK (execution_time_ms >= 0)
);

-- Indexes for performance
CREATE INDEX idx_skills_agent ON agent_skills(agent_id);
CREATE INDEX idx_skills_public ON agent_skills(is_public) WHERE is_public = TRUE;
CREATE INDEX idx_skills_verified ON agent_skills(is_verified) WHERE is_verified = TRUE;
CREATE INDEX idx_skills_downloads ON agent_skills(downloads DESC);
CREATE INDEX idx_skills_name ON agent_skills(skill_name);
CREATE INDEX idx_skill_installations_agent ON agent_skill_installations(agent_id);
CREATE INDEX idx_skill_installations_skill ON agent_skill_installations(skill_id);
CREATE INDEX idx_skill_logs_agent ON agent_skill_logs(agent_id, created_at DESC);
CREATE INDEX idx_skill_logs_skill ON agent_skill_logs(skill_id, created_at DESC);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_agent_skills_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER agent_skills_update_timestamp
    BEFORE UPDATE ON agent_skills
    FOR EACH ROW
    EXECUTE FUNCTION update_agent_skills_timestamp();

-- Comments for documentation
COMMENT ON TABLE agent_skills IS 'Stores AI agent skills and code';
COMMENT ON TABLE agent_skill_installations IS 'Tracks which agents have installed which skills';
COMMENT ON TABLE agent_skill_logs IS 'Audit trail of skill executions';
COMMENT ON COLUMN agent_skills.skill_type IS '0=Builtin, 1=Custom, 2=Shared, 3=External';
COMMENT ON COLUMN agent_skills.skill_code IS 'Executable code for the skill (security scanned)';
COMMENT ON COLUMN agent_skills.is_verified IS 'Whether skill has passed security verification';
