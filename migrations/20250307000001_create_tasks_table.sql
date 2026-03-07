CREATE TABLE tasks (
    id TEXT PRIMARY KEY,  -- UUID v4 as string
    prompt TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('pending', 'running', 'completed', 'failed')),
    response TEXT,        -- Agent's response/summary
    exit_code INTEGER,
    created_at TEXT NOT NULL,  -- RFC 3339 UTC timestamp
    started_at TEXT,
    completed_at TEXT
);

CREATE INDEX idx_status_created ON tasks(status, created_at);
