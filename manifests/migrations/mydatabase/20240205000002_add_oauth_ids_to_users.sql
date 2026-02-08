-- Add OAuth provider IDs to users table for backward compatibility
-- The oauth_accounts table handles detailed OAuth data, but these columns
-- provide quick lookup for OAuth login flow

ALTER TABLE users ADD COLUMN google_id VARCHAR(255);
ALTER TABLE users ADD COLUMN github_id VARCHAR(255);

CREATE UNIQUE INDEX idx_users_google_id ON users(google_id) WHERE google_id IS NOT NULL;
CREATE UNIQUE INDEX idx_users_github_id ON users(github_id) WHERE github_id IS NOT NULL;
