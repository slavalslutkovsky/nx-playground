-- TimescaleDB Setup for Time-Series Data
-- Run this after installing TimescaleDB extension

-- Enable TimescaleDB
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Convert cost_data to hypertable for efficient time-series queries
SELECT create_hypertable('cost_data', 'usage_start',
    chunk_time_interval => INTERVAL '7 days',
    if_not_exists => TRUE
);

-- Add compression policy (compress data older than 30 days)
ALTER TABLE cost_data SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'organization_id,provider',
    timescaledb.compress_orderby = 'usage_start DESC'
);

SELECT add_compression_policy('cost_data', INTERVAL '30 days');

-- Add retention policy (keep data for 2 years)
SELECT add_retention_policy('cost_data', INTERVAL '2 years');

-- Create continuous aggregates for common queries
CREATE MATERIALIZED VIEW cost_data_daily
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 day', usage_start) AS day,
    organization_id,
    provider,
    service,
    SUM(amount) AS total_cost,
    COUNT(*) AS record_count
FROM cost_data
GROUP BY day, organization_id, provider, service
WITH NO DATA;

SELECT add_continuous_aggregate_policy('cost_data_daily',
    start_offset => INTERVAL '3 days',
    end_offset => INTERVAL '1 day',
    schedule_interval => INTERVAL '1 hour');

-- Create monthly aggregate view
CREATE MATERIALIZED VIEW cost_data_monthly
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 month', usage_start) AS month,
    organization_id,
    provider,
    service,
    SUM(amount) AS total_cost,
    COUNT(*) AS record_count
FROM cost_data
GROUP BY month, organization_id, provider, service
WITH NO DATA;

SELECT add_continuous_aggregate_policy('cost_data_monthly',
    start_offset => INTERVAL '3 months',
    end_offset => INTERVAL '1 month',
    schedule_interval => INTERVAL '1 day');
