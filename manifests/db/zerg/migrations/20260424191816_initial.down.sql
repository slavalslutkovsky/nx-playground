-- Revert initial schema

DROP TABLE IF EXISTS cloud_resources CASCADE;
DROP TABLE IF EXISTS tasks CASCADE;
DROP TABLE IF EXISTS projects CASCADE;
DROP TABLE IF EXISTS oauth_accounts CASCADE;
DROP TABLE IF EXISTS users CASCADE;

DROP TYPE IF EXISTS resource_status;
DROP TYPE IF EXISTS resource_type;
DROP TYPE IF EXISTS project_status;
DROP TYPE IF EXISTS environment;
DROP TYPE IF EXISTS cloud_provider;
DROP TYPE IF EXISTS task_status;
DROP TYPE IF EXISTS task_priority;

DROP FUNCTION IF EXISTS util.touch_updated_at();
DROP SCHEMA IF EXISTS util;

-- Extensions are intentionally left in place (pgcrypto, hstore) — they may be
-- used by other schemas or applications sharing the database.
