-- Create sessions table for agent session tracking
CREATE TABLE IF NOT EXISTS sessions (
    id BLOB PRIMARY KEY,
    agent_id TEXT NOT NULL,
    workspace_path TEXT NOT NULL,
    prompt TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    result TEXT,
    error TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at DATETIME,
    finished_at DATETIME,
    priority INTEGER NOT NULL DEFAULT 0
);

-- Index for querying sessions by status
CREATE INDEX idx_sessions_status ON sessions(status);

-- Index for querying sessions by agent_id
CREATE INDEX idx_sessions_agent_id ON sessions(agent_id);

-- Index for querying sessions by priority (for scheduling)
CREATE INDEX idx_sessions_priority ON sessions(priority DESC, created_at ASC);

-- Index for querying pending sessions ordered by creation time
CREATE INDEX idx_sessions_created_at ON sessions(created_at);
