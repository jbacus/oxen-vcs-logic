-- Create lock status enum
CREATE TYPE lock_status AS ENUM ('active', 'expired', 'released');

-- Create locks table
CREATE TABLE IF NOT EXISTS locks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    lock_id VARCHAR(255) NOT NULL,
    project_path TEXT NOT NULL,
    locked_by_user_id UUID NOT NULL REFERENCES users(id),
    locked_by_identifier VARCHAR(255) NOT NULL,
    machine_id VARCHAR(255) NOT NULL,
    acquired_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_heartbeat TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status lock_status NOT NULL DEFAULT 'active',
    released_at TIMESTAMPTZ,

    UNIQUE(repository_id, project_path)
);

CREATE INDEX idx_locks_repository_id ON locks(repository_id);
CREATE INDEX idx_locks_status ON locks(status) WHERE status = 'active';
CREATE INDEX idx_locks_expires_at ON locks(expires_at) WHERE status = 'active';
CREATE INDEX idx_locks_user_id ON locks(locked_by_user_id);
