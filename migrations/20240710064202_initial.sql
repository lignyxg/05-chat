-- Add migration script here

-- create user table
CREATE TABLE IF NOT EXISTS users(
    id bigserial PRIMARY KEY,
    ws_id bigint NOT NULL,
    fullname VARCHAR(255) NOT NULL,
    email varchar(255) NOT NULL UNIQUE,
    password_hash varchar(255) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT NOW(),
    updated_at timestamptz NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workspaces(
    id bigserial PRIMARY KEY,
    name VARCHAR(64) NOT NULL UNIQUE,
    owner_id bigint NOT NULL REFERENCES users(id),
    created_at timestamptz NOT NULL DEFAULT NOW(),
    updated_at timestamptz NOT NULL DEFAULT NOW()
);

-- create index for users for email
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- insert super admin user
INSERT INTO users(id, ws_id, fullname, email, password_hash)
VALUES (0, 0, 'Super Admin', '4qLrX@example.com', 'superadmin');

-- insert super admin workspace
INSERT INTO workspaces(id, name, owner_id)
VALUES (0, 'Super Admin Workspace', 0);


-- create chat type: single, group, private_channel, public_channel
CREATE TYPE chat_type AS ENUM('single', 'group', 'private_channel', 'public_channel');

-- create chat table
CREATE TABLE IF NOT EXISTS chats(
    id bigserial PRIMARY KEY,
    ws_id bigint NOT NULL REFERENCES workspaces(id),
    owner_id bigint,
    name VARCHAR(255),
    type chat_type NOT NULL,
    members bigint[] NOT NULL,
    created_at timestamptz NOT NULL DEFAULT NOW(),
    updated_at timestamptz NOT NULL DEFAULT NOW()
);

-- create messages table
CREATE TABLE IF NOT EXISTS messages(
    id bigserial PRIMARY KEY,
    chat_id bigint NOT NULL REFERENCES chats(id),
    sender_id bigint NOT NULL REFERENCES users(id),
    content text NOT NULL,
    file text[] NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT NOW()
);

-- create index for messages for chat_id and created_at order by created_at desc
CREATE INDEX IF NOT EXISTS idx_messages_chat_id_created_at ON messages(chat_id, created_at DESC);

-- create index for messages for sender_id and created_at order by created_at desc
CREATE INDEX IF NOT EXISTS idx_messages_sender_id_created_at ON messages(sender_id, created_at DESC);

-- insert workspace
INSERT INTO workspaces(name, owner_id)
VALUES ('bbc', 0),
    ('cnn', 0),
    ('fox', 0);


-- insert user
INSERT INTO users(ws_id, fullname, email, password_hash)
VALUES (1, 'tyran', 'tyran@bbc.com', '$argon2id$v=19$m=19456,t=2,p=1$9aOtUTcJy/yAAAfJnx7oyg$KtrEx3fkMTwSwyl8oFub53Zzg5YCMVIQbmLhIdwJkAU'),
    (1, 'alice', 'alice@bbc.com', '$argon2id$v=19$m=19456,t=2,p=1$9aOtUTcJy/yAAAfJnx7oyg$KtrEx3fkMTwSwyl8oFub53Zzg5YCMVIQbmLhIdwJkAU'),
    (1, 'bob', 'bob@bbc.com', '$argon2id$v=19$m=19456,t=2,p=1$9aOtUTcJy/yAAAfJnx7oyg$KtrEx3fkMTwSwyl8oFub53Zzg5YCMVIQbmLhIdwJkAU'),
    (1, 'charlie', 'charlie@bbc.com', '$argon2id$v=19$m=19456,t=2,p=1$9aOtUTcJy/yAAAfJnx7oyg$KtrEx3fkMTwSwyl8oFub53Zzg5YCMVIQbmLhIdwJkAU'),
    (3, 'doe', 'doe@fox.com', '$argon2id$v=19$m=19456,t=2,p=1$9aOtUTcJy/yAAAfJnx7oyg$KtrEx3fkMTwSwyl8oFub53Zzg5YCMVIQbmLhIdwJkAU'),
    (3, 'eve', 'eve@fox.com', '$argon2id$v=19$m=19456,t=2,p=1$9aOtUTcJy/yAAAfJnx7oyg$KtrEx3fkMTwSwyl8oFub53Zzg5YCMVIQbmLhIdwJkAU');


-- insert named chats
INSERT INTO chats(ws_id, owner_id, name, type, members)
VALUES (1, 1, 'group_chat', 'group', '{1, 2, 3, 4}'),
    (1, 2, 'private_ch', 'private_channel', '{1, 2, 3}'),
    (1, 3, 'general_ch', 'public_channel', '{1, 2, 3, 4, 5, 6}');

-- insert unnamed chat
INSERT INTO chats(ws_id, type, members)
VALUES (1, 'single', '{1, 2}'), (1, 'single', '{2, 3}');

-- insert messages
INSERT INTO messages(chat_id, sender_id, content, file)
VALUES (1, 1, 'hello', '{}'),
    (1, 2, 'hello world', '{}'),
    (1, 3, 'rust and go', '{}'),
    (1, 4, 'hello', '{}'),
    (2, 1, 'how are you', '{}'),
    (2, 2, 'yes', '{}'),
    (2, 3, 'ok but i am fine', '{}'),
    (3, 1, 'hello', '{}'),
    (3, 2, 'hello', '{}'),
    (3, 3, 'hello', '{}'),
    (3, 4, 'hello', '{}'),
    (3, 5, 'hello', '{}'),
    (3, 6, 'hello', '{}');