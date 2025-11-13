CREATE TABLE IF NOT EXISTS hosts (
    id UUID PRIMARY KEY,
    network_id UUID NOT NULL REFERENCES networks(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    hostname TEXT,
    description TEXT,
    target JSONB NOT NULL,
    interfaces JSONB,
    services JSONB,
    ports JSONB,
    source JSONB NOT NULL,
    virtualization JSONB,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_hosts_network ON hosts(network_id);