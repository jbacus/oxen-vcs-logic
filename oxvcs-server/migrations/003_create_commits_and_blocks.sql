-- Create commits table
CREATE TABLE IF NOT EXISTS commits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    commit_hash VARCHAR(64) NOT NULL,
    parent_hash VARCHAR(64),
    author_name VARCHAR(255) NOT NULL,
    author_email VARCHAR(255) NOT NULL,
    committer_name VARCHAR(255) NOT NULL,
    committer_email VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    branch VARCHAR(255) NOT NULL DEFAULT 'main',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Logic Pro specific metadata
    bpm DECIMAL(6,2),
    sample_rate INTEGER,
    key_signature VARCHAR(50),
    tags TEXT[],

    UNIQUE(repository_id, commit_hash)
);

CREATE INDEX idx_commits_repository_id ON commits(repository_id);
CREATE INDEX idx_commits_commit_hash ON commits(commit_hash);
CREATE INDEX idx_commits_branch ON commits(branch);
CREATE INDEX idx_commits_created_at ON commits(created_at DESC);
CREATE INDEX idx_commits_parent_hash ON commits(parent_hash);

-- Create branches table
CREATE TABLE IF NOT EXISTS branches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    head_commit_hash VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(repository_id, name)
);

CREATE INDEX idx_branches_repository_id ON branches(repository_id);
CREATE INDEX idx_branches_name ON branches(name);

CREATE TRIGGER update_branches_updated_at
    BEFORE UPDATE ON branches
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create blocks table (content-addressed storage)
CREATE TABLE IF NOT EXISTS blocks (
    hash VARCHAR(64) PRIMARY KEY,
    size_bytes BIGINT NOT NULL,
    storage_key TEXT NOT NULL,
    ref_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_blocks_ref_count ON blocks(ref_count);
CREATE INDEX idx_blocks_created_at ON blocks(created_at);

-- Create commit_blocks junction table
CREATE TABLE IF NOT EXISTS commit_blocks (
    commit_id UUID NOT NULL REFERENCES commits(id) ON DELETE CASCADE,
    block_hash VARCHAR(64) NOT NULL REFERENCES blocks(hash) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    block_offset BIGINT NOT NULL,

    PRIMARY KEY (commit_id, block_hash, block_offset)
);

CREATE INDEX idx_commit_blocks_commit_id ON commit_blocks(commit_id);
CREATE INDEX idx_commit_blocks_block_hash ON commit_blocks(block_hash);
