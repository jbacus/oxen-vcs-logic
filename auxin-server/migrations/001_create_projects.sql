-- Create projects table
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    namespace TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    repository_path TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(namespace, name)
);

-- Create index on namespace for faster lookups
CREATE INDEX IF NOT EXISTS idx_projects_namespace ON projects(namespace);

-- Create index on name for faster searches
CREATE INDEX IF NOT EXISTS idx_projects_name ON projects(name);
