# Postgres Migration Guidelines

- Run schema changes inside a transaction (`BEGIN; … COMMIT;`).
- Manage shared helpers in `0000_bootstrap.sql` (extensions, utility functions).
- Keep migrations schema-only; seed data lives in `seeds/`.
- Prefer UUID primary keys via `gen_random_uuid()` and ensure `pgcrypto` is installed.
- Use `GENERATED … AS IDENTITY` instead of legacy `serial` for integer identifiers.
- Namespace reusable trigger functions under `util` to avoid collisions.
- When notifying listeners, emit payloads that include `TG_OP` and guard `NEW`/`OLD`.
- Pair schema migrations with optional down notes or rollback scripts where needed.
