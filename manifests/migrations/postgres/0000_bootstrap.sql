BEGIN;

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE SCHEMA IF NOT EXISTS util;

CREATE OR REPLACE FUNCTION util.touch_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION util.notify_table_events()
RETURNS TRIGGER AS $$
DECLARE
    payload JSONB;
BEGIN
    payload := jsonb_build_object(
        'table', TG_TABLE_NAME,
        'action', TG_OP,
        'data', CASE
            WHEN TG_OP IN ('INSERT', 'UPDATE') THEN to_jsonb(NEW)
            ELSE NULL
        END,
        'prev_data', CASE
            WHEN TG_OP IN ('UPDATE', 'DELETE') THEN to_jsonb(OLD)
            ELSE NULL
        END
    );

    PERFORM pg_notify('table_events', payload::text);

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

COMMIT;
