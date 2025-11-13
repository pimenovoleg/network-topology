CREATE TABLE IF NOT EXISTS services (
    id UUID PRIMARY KEY,
    network_id UUID NOT NULL REFERENCES networks(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    name TEXT NOT NULL,
    host_id UUID NOT NULL,
    bindings JSONB,
    service_definition TEXT NOT NULL,
    virtualization JSONB,
    source JSONB NOT NULL,
    FOREIGN KEY (host_id) REFERENCES hosts(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_services_host_id ON services(host_id);
CREATE INDEX IF NOT EXISTS idx_services_network ON services(network_id);