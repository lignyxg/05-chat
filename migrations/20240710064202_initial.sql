-- Add migration script here

-- create user table
CREATE TABLE IF NOT EXISTS users(
    id bigserial PRIMARY KEY,
    fullname VARCHAR(255) NOT NULL,
    email varchar(255) NOT NULL UNIQUE,
    password_hash varchar(255) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT NOW(),
    updated_at timestamptz NOT NULL DEFAULT NOW()
);

-- create index for users for email
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- create chat type: single, group, private_channel, public_channel
CREATE TYPE chat_type AS ENUM('single', 'group', 'private_channel', 'public_channel');

-- create chat table
CREATE TABLE IF NOT EXISTS chats(
    id bigserial PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
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
    created_at timestamptz NOT NULL DEFAULT NOW()
);

-- create index for messages for chat_id and created_at order by created_at desc
CREATE INDEX IF NOT EXISTS idx_messages_chat_id_created_at ON messages(chat_id, created_at DESC);

-- create index for messages for sender_id and created_at order by created_at desc
CREATE INDEX IF NOT EXISTS idx_messages_sender_id_created_at ON messages(sender_id, created_at DESC);
