-- Add migration script here
CREATE TABLE IF NOT EXISTS workspaces(
    id bigserial PRIMARY KEY,
    name VARCHAR(64) NOT NULL UNIQUE,
    owner_id bigint NOT NULL REFERENCES users(id),
    created_at timestamptz NOT NULL DEFAULT NOW(),
    updated_at timestamptz NOT NULL DEFAULT NOW()
);

-- alter table users to add ws_id
ALTER TABLE users ADD COLUMN ws_id bigint;

-- alter table chats to add ws_id
ALTER TABLE chats ADD COLUMN ws_id bigint;

-- alter table chats to add owner_id
ALTER TABLE chats ADD COLUMN owner_id bigint;


-- insert super admin user
INSERT INTO users(id, ws_id, fullname, email, password_hash)
VALUES (0, 0, 'Super Admin', '4qLrX@example.com', 'superadmin');

-- insert super admin workspace
INSERT INTO workspaces(id, name, owner_id)
VALUES (0, 'Super Admin Workspace', 0);
