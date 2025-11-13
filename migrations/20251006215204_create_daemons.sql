CREATE TABLE IF NOT EXISTS daemons (
    id UUID PRIMARY KEY,
    network_id UUID NOT NULL REFERENCES networks(id) ON DELETE CASCADE,
    host_id UUID NOT NULL,
    ip TEXT NOT NULL,
    port INTEGER NOT NULL,
    registered_at TIMESTAMPTZ NOT NULL,
    last_seen TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_daemon_host_id ON daemons(host_id);
CREATE INDEX IF NOT EXISTS idx_daemons_network ON daemons(network_id);