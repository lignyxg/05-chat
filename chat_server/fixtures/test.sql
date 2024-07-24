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