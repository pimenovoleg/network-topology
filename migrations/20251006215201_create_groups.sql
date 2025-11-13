CREATE TABLE IF NOT EXISTS groups (
    id UUID PRIMARY KEY,
    network_id UUID NOT NULL REFERENCES networks(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    group_type JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    source JSONB NOT NULL,
    color TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_groups_network ON groups(network_id);