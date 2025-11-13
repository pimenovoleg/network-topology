CREATE TABLE IF NOT EXISTS subnets (
    id UUID PRIMARY KEY,
    network_id UUID NOT NULL REFERENCES networks(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    cidr TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    subnet_type TEXT NOT NULL,
    source JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_subnets_network ON subnets(network_id);