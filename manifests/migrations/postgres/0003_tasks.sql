BEGIN;

CREATE TABLE IF NOT EXISTS tasks (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  title TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  completed BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TRIGGER tasks_emit_table_event
  AFTER INSERT OR UPDATE OR DELETE ON tasks
  FOR EACH ROW EXECUTE FUNCTION util.notify_table_events();

COMMIT;
