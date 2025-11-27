-- FinOps Multi-Cloud Cost Optimization Platform
-- Initial Schema Migration

-- Enable extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Organizations table
CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_organizations_name ON organizations(name);

-- Users table (for multi-tenant access control)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255),
    role VARCHAR(50) NOT NULL DEFAULT 'viewer',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_organization ON users(organization_id);
CREATE INDEX idx_users_email ON users(email);

-- Cloud credentials table (encrypted sensitive data)
CREATE TABLE cloud_credentials (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,
    credential_type VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    encrypted_data BYTEA NOT NULL,
    encryption_key_id VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_validated TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_cloud_credentials_org ON cloud_credentials(organization_id);
CREATE INDEX idx_cloud_credentials_provider ON cloud_credentials(provider);

-- Cloud resources table
CREATE TABLE cloud_resources (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,
    resource_type VARCHAR(255) NOT NULL,
    resource_id VARCHAR(512) NOT NULL,
    name VARCHAR(512) NOT NULL,
    region VARCHAR(100) NOT NULL,
    tags JSONB,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, provider, resource_id)
);

CREATE INDEX idx_resources_org ON cloud_resources(organization_id);
CREATE INDEX idx_resources_provider ON cloud_resources(provider);
CREATE INDEX idx_resources_type ON cloud_resources(resource_type);
CREATE INDEX idx_resources_tags ON cloud_resources USING GIN(tags);

-- Cost data table (time-series, consider TimescaleDB hypertable)
CREATE TABLE cost_data (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    resource_id UUID REFERENCES cloud_resources(id) ON DELETE SET NULL,
    provider VARCHAR(50) NOT NULL,
    service VARCHAR(255) NOT NULL,
    amount DECIMAL(15, 4) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    usage_start TIMESTAMPTZ NOT NULL,
    usage_end TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_cost_data_org ON cost_data(organization_id);
CREATE INDEX idx_cost_data_resource ON cost_data(resource_id);
CREATE INDEX idx_cost_data_usage_start ON cost_data(usage_start DESC);
CREATE INDEX idx_cost_data_provider ON cost_data(provider);

-- Recommendations table
CREATE TABLE recommendations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    resource_id UUID REFERENCES cloud_resources(id) ON DELETE SET NULL,
    recommendation_type VARCHAR(100) NOT NULL,
    current_provider VARCHAR(50) NOT NULL,
    recommended_provider VARCHAR(50),
    title VARCHAR(512) NOT NULL,
    description TEXT NOT NULL,
    reasoning TEXT NOT NULL,
    estimated_savings DECIMAL(15, 4) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    confidence_score DECIMAL(3, 2) NOT NULL,
    migration_complexity VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_recommendations_org ON recommendations(organization_id);
CREATE INDEX idx_recommendations_resource ON recommendations(resource_id);
CREATE INDEX idx_recommendations_status ON recommendations(status);
CREATE INDEX idx_recommendations_created ON recommendations(created_at DESC);

-- Audit log table
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100),
    resource_id UUID,
    details JSONB,
    ip_address INET,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_org ON audit_logs(organization_id);
CREATE INDEX idx_audit_logs_created ON audit_logs(created_at DESC);

-- Row-level security for multi-tenancy
ALTER TABLE organizations ENABLE ROW LEVEL SECURITY;
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE cloud_credentials ENABLE ROW LEVEL SECURITY;
ALTER TABLE cloud_resources ENABLE ROW LEVEL SECURITY;
ALTER TABLE cost_data ENABLE ROW LEVEL SECURITY;
ALTER TABLE recommendations ENABLE ROW LEVEL SECURITY;

-- Functions for updated_at trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers
CREATE TRIGGER update_organizations_updated_at BEFORE UPDATE ON organizations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_cloud_credentials_updated_at BEFORE UPDATE ON cloud_credentials
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_cloud_resources_updated_at BEFORE UPDATE ON cloud_resources
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_recommendations_updated_at BEFORE UPDATE ON recommendations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Sample data for development
INSERT INTO organizations (name) VALUES ('Demo Organization');
