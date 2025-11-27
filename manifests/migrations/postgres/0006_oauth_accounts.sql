BEGIN;

CREATE TABLE IF NOT EXISTS oauth_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    provider_username VARCHAR(255),
    email VARCHAR(255),
    display_name VARCHAR(255),
    avatar_url TEXT,
    access_token TEXT,
    refresh_token TEXT,
    token_expires_at TIMESTAMPTZ,
    scopes TEXT[],
    raw_user_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(provider, provider_user_id)
);

CREATE INDEX IF NOT EXISTS idx_oauth_accounts_user_id ON oauth_accounts(user_id);
CREATE INDEX IF NOT EXISTS idx_oauth_accounts_provider ON oauth_accounts(provider);
CREATE INDEX IF NOT EXISTS idx_oauth_accounts_email ON oauth_accounts(email);

CREATE TABLE IF NOT EXISTS oauth_link_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    provider_data JSONB NOT NULL,
    token VARCHAR(64) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_oauth_link_requests_token ON oauth_link_requests(token);
CREATE INDEX IF NOT EXISTS idx_oauth_link_requests_expires ON oauth_link_requests(expires_at);

ALTER TABLE users ALTER COLUMN password_hash DROP NOT NULL;

ALTER TABLE users ADD COLUMN IF NOT EXISTS email_verified BOOLEAN NOT NULL DEFAULT FALSE;

CREATE TRIGGER oauth_accounts_touch_updated_at
    BEFORE UPDATE ON oauth_accounts
    FOR EACH ROW
    EXECUTE FUNCTION util.touch_updated_at();

COMMIT;
