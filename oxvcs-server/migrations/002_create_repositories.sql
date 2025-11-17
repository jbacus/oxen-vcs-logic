-- Create repository visibility enum
CREATE TYPE repository_visibility AS ENUM ('public', 'private', 'internal');

-- Create repositories table
CREATE TABLE IF NOT EXISTS repositories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    namespace VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    visibility repository_visibility NOT NULL DEFAULT 'private',
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    storage_path TEXT NOT NULL,
    default_branch VARCHAR(255) NOT NULL DEFAULT 'main',
    size_bytes BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_push_at TIMESTAMPTZ,
    is_archived BOOLEAN NOT NULL DEFAULT FALSE,

    UNIQUE(namespace, name),
    CONSTRAINT name_length CHECK (char_length(name) >= 1 AND char_length(name) <= 100),
    CONSTRAINT name_format CHECK (name ~* '^[a-z0-9][a-z0-9-]*[a-z0-9]$')
);

CREATE INDEX idx_repositories_owner_id ON repositories(owner_id);
CREATE INDEX idx_repositories_namespace ON repositories(namespace);
CREATE INDEX idx_repositories_visibility ON repositories(visibility);
CREATE INDEX idx_repositories_created_at ON repositories(created_at DESC);

CREATE TRIGGER update_repositories_updated_at
    BEFORE UPDATE ON repositories
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create access level enum
CREATE TYPE access_level AS ENUM ('none', 'read', 'write', 'admin', 'owner');

-- Create repository collaborators table
CREATE TABLE IF NOT EXISTS repository_collaborators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_level access_level NOT NULL DEFAULT 'read',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    invited_by UUID REFERENCES users(id),

    UNIQUE(repository_id, user_id)
);

CREATE INDEX idx_repo_collaborators_repo_id ON repository_collaborators(repository_id);
CREATE INDEX idx_repo_collaborators_user_id ON repository_collaborators(user_id);
