BEGIN;

INSERT INTO books (id, title, author, description, is_published, copies_sold)
VALUES (
    '11111111-1111-4111-8111-111111111111',
    'Moby Dick',
    'Herman Melville',
    'A sea captain obsessively hunts a great white whale.',
    FALSE,
    0
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO todos (id, title, completed)
VALUES
    (1, 'Hello, world!', FALSE),
    (2, 'Clean room', FALSE)
ON CONFLICT (id) DO NOTHING;

SELECT
    setval(
        pg_get_serial_sequence('todos', 'id'),
        COALESCE((SELECT MAX(id) FROM todos), 1),
        TRUE
    );

INSERT INTO tasks (id, title, description, completed)
VALUES
    ('22222222-2222-4222-8222-222222222222', 'Hello, world!', 'My task desc', FALSE),
    ('33333333-3333-4333-8333-333333333333', 'Clean room', 'My task desc', FALSE),
    ('44444444-4444-4444-8444-444444444444', 'Present on Rust', 'Rust app for CLI and Webserver', TRUE)
ON CONFLICT (id) DO NOTHING;

INSERT INTO projects (id, title, description, completed)
VALUES
    ('55555555-5555-4555-8555-555555555555', 'Playground Monorepo', 'Main development playground', FALSE),
    ('66666666-6666-4666-8666-666666666666', 'Terran API', 'API service for terran application', TRUE),
    ('77777777-7777-4777-8777-777777777777', 'KCL Modules', 'Kubernetes Configuration Language modules', FALSE)
ON CONFLICT (id) DO NOTHING;

COMMIT;
