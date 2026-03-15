-- Agent Collaboration Workspaces
-- Migration: Create workspace tables for agent collaboration

-- ============================================================================
-- Agent Workspaces Table
-- ============================================================================

CREATE TABLE agent_workspaces (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    owner_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    is_public BOOLEAN NOT NULL DEFAULT false,
    max_members INTEGER NOT NULL DEFAULT 10,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_agent_workspaces_owner ON agent_workspaces(owner_id);
CREATE INDEX idx_agent_workspaces_created_at ON agent_workspaces(created_at DESC);
CREATE INDEX idx_agent_workspaces_is_public ON agent_workspaces(is_public);

-- ============================================================================
-- Workspace Members Table
-- ============================================================================

CREATE TABLE agent_workspace_members (
    id SERIAL PRIMARY KEY,
    workspace_id INTEGER NOT NULL REFERENCES agent_workspaces(id) ON DELETE CASCADE,
    agent_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    role INTEGER NOT NULL DEFAULT 2, -- 0=Owner, 1=Admin, 2=Member, 3=Viewer
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(workspace_id, agent_id)
);

-- Indexes
CREATE INDEX idx_workspace_members_workspace ON agent_workspace_members(workspace_id);
CREATE INDEX idx_workspace_members_agent ON agent_workspace_members(agent_id);
CREATE INDEX idx_workspace_members_role ON agent_workspace_members(role);
CREATE INDEX idx_workspace_members_last_active ON agent_workspace_members(last_active DESC);

-- ============================================================================
-- Workspace Tasks Table
-- ============================================================================

CREATE TABLE agent_workspace_tasks (
    id SERIAL PRIMARY KEY,
    workspace_id INTEGER NOT NULL REFERENCES agent_workspaces(id) ON DELETE CASCADE,
    title VARCHAR(200) NOT NULL,
    description TEXT,
    status INTEGER NOT NULL DEFAULT 0, -- 0=Todo, 1=InProgress, 2=Review, 3=Done, 4=Cancelled
    priority INTEGER NOT NULL DEFAULT 1, -- 0=Low, 1=Medium, 2=High, 3=Critical
    assigned_to INTEGER REFERENCES person(id) ON DELETE SET NULL,
    created_by INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    due_date TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- Indexes
CREATE INDEX idx_workspace_tasks_workspace ON agent_workspace_tasks(workspace_id);
CREATE INDEX idx_workspace_tasks_status ON agent_workspace_tasks(status);
CREATE INDEX idx_workspace_tasks_priority ON agent_workspace_tasks(priority);
CREATE INDEX idx_workspace_tasks_assigned_to ON agent_workspace_tasks(assigned_to);
CREATE INDEX idx_workspace_tasks_created_by ON agent_workspace_tasks(created_by);
CREATE INDEX idx_workspace_tasks_due_date ON agent_workspace_tasks(due_date);
CREATE INDEX idx_workspace_tasks_created_at ON agent_workspace_tasks(created_at DESC);

-- ============================================================================
-- Workspace Activities Table
-- ============================================================================

CREATE TABLE agent_workspace_activities (
    id SERIAL PRIMARY KEY,
    workspace_id INTEGER NOT NULL REFERENCES agent_workspaces(id) ON DELETE CASCADE,
    agent_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    activity_type INTEGER NOT NULL, -- 0=Created, 1=MemberAdded, 2=MemberRemoved, 3=TaskCreated, etc.
    target_id INTEGER, -- ID of affected entity
    description TEXT NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_workspace_activities_workspace ON agent_workspace_activities(workspace_id);
CREATE INDEX idx_workspace_activities_agent ON agent_workspace_activities(agent_id);
CREATE INDEX idx_workspace_activities_type ON agent_workspace_activities(activity_type);
CREATE INDEX idx_workspace_activities_created_at ON agent_workspace_activities(created_at DESC);
CREATE INDEX idx_workspace_activities_metadata ON agent_workspace_activities USING gin(metadata);

-- ============================================================================
-- Triggers
-- ============================================================================

-- Auto-update updated_at timestamp for workspaces
CREATE OR REPLACE FUNCTION update_workspace_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_workspace_timestamp
    BEFORE UPDATE ON agent_workspaces
    FOR EACH ROW
    EXECUTE FUNCTION update_workspace_timestamp();

-- Auto-update updated_at timestamp for tasks
CREATE TRIGGER trigger_update_task_timestamp
    BEFORE UPDATE ON agent_workspace_tasks
    FOR EACH ROW
    EXECUTE FUNCTION update_workspace_timestamp();

-- ============================================================================
-- Comments
-- ============================================================================

COMMENT ON TABLE agent_workspaces IS 'Collaborative workspaces for agent teams';
COMMENT ON TABLE agent_workspace_members IS 'Members of workspaces with roles';
COMMENT ON TABLE agent_workspace_tasks IS 'Tasks within workspaces';
COMMENT ON TABLE agent_workspace_activities IS 'Activity log for workspace events';

COMMENT ON COLUMN agent_workspaces.owner_id IS 'Agent who owns the workspace';
COMMENT ON COLUMN agent_workspaces.is_public IS 'Whether workspace is publicly visible';
COMMENT ON COLUMN agent_workspaces.max_members IS 'Maximum number of members allowed';

COMMENT ON COLUMN agent_workspace_members.role IS '0=Owner, 1=Admin, 2=Member, 3=Viewer';
COMMENT ON COLUMN agent_workspace_members.last_active IS 'Last activity timestamp in workspace';

COMMENT ON COLUMN agent_workspace_tasks.status IS '0=Todo, 1=InProgress, 2=Review, 3=Done, 4=Cancelled';
COMMENT ON COLUMN agent_workspace_tasks.priority IS '0=Low, 1=Medium, 2=High, 3=Critical';
COMMENT ON COLUMN agent_workspace_tasks.completed_at IS 'Timestamp when task was completed';
