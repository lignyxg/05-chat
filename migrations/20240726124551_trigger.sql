-- Add migration script here
-- if chat updated, notify with that data
CREATE OR REPLACE FUNCTION add_to_chat() RETURNS TRIGGER AS $$
BEGIN
    RAISE NOTICE 'add_to_chat:%', NEW;
    PERFORM pg_notify('chat_update', json_build_object(
        'op', TG_OP,
        'old', OLD,
        'new', NEW
    )::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER add_to_chat_trigger
    AFTER INSERT OR UPDATE OR DELETE
    ON chats
    FOR EACH ROW
    EXECUTE PROCEDURE add_to_chat();

-- if new messages added, notify with that data
CREATE OR REPLACE FUNCTION add_to_messages() RETURNS TRIGGER AS $$
DECLARE
users bigint[];
BEGIN
    RAISE NOTICE 'add_to_messages:%', NEW;
    SELECT members INTO users FROM chats WHERE id = NEW.chat_id;
    PERFORM pg_notify('messages_create', json_build_object(
        'messages', NEW,
        'users', users
    )::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER add_to_messages_trigger
    AFTER INSERT OR UPDATE
    ON messages
    FOR EACH ROW
    EXECUTE PROCEDURE add_to_messages();