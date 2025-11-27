BEGIN;

CREATE TABLE IF NOT EXISTS books (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  title TEXT NOT NULL,
  author TEXT NOT NULL,
  description TEXT NOT NULL,
  is_published BOOLEAN NOT NULL DEFAULT FALSE,
  copies_sold INTEGER NOT NULL DEFAULT 0 CHECK (copies_sold >= 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_books_title ON books (title);

COMMIT;
