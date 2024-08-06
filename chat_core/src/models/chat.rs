use sqlx::{query_as, PgPool};
use tracing::info;

use crate::error::ChatCoreError;
use crate::models::{Chat, ChatType, CreateChat, User};

impl Chat {
    pub async fn create(
        create_chat: CreateChat,
        ws_id: i64,
        user_id: i64,
        pool: &PgPool,
    ) -> Result<Self, ChatCoreError> {
        if !create_chat.members.contains(&user_id) {
            return Err(ChatCoreError::CreateChatError(
                "User not in chat members".to_string(),
            ));
        }
        let len = create_chat.members.len();
        if len < 2 {
            return Err(ChatCoreError::CreateChatError(
                "Chat must have at least 2 members".to_string(),
            ));
        }
        if len > 8 && create_chat.name.is_none() {
            return Err(ChatCoreError::CreateChatError(
                "Chat with more than 8 members must have a name".to_string(),
            ));
        }
        let users = User::find_user_by_ids(&create_chat.members, pool).await?;
        if users.len() != len {
            return Err(ChatCoreError::CreateChatError(
                "Some Chat members not found".to_string(),
            ));
        }
        let typ = match len {
            2 => ChatType::Single,
            3..=8 => ChatType::Group,
            _ if create_chat.is_public => ChatType::PublicChannel,
            _ => ChatType::PrivateChannel,
        };

        let owner_id = if typ == ChatType::Single {
            None
        } else {
            Some(user_id)
        };

        let chat = query_as(
            r#"
            INSERT INTO chats (ws_id, owner_id, name, type, members)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(ws_id)
        .bind(owner_id)
        .bind(create_chat.name)
        .bind(typ)
        .bind(create_chat.members)
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }

    pub(crate) async fn find_chat_by_id(
        id: i64,
        pool: &PgPool,
    ) -> Result<Option<Self>, ChatCoreError> {
        let chat = query_as(
            r#"
            SELECT *
            FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(chat)
    }
    pub async fn list_chats_in_workspace(
        ws_id: i64,
        pool: &PgPool,
    ) -> Result<Vec<Self>, ChatCoreError> {
        let chats = query_as(
            r#"
            SELECT *
            FROM chats
            WHERE ws_id = $1
            "#,
        )
        .bind(ws_id)
        .fetch_all(pool)
        .await?;

        Ok(chats)
    }

    pub async fn delete(id: i64, user_id: i64, pool: &PgPool) -> Result<Self, ChatCoreError> {
        let chat = Chat::find_chat_by_id(id, pool).await?;
        match chat {
            Some(chat) => {
                if chat.typ != ChatType::Single && chat.owner_id != Some(user_id) {
                    return Err(ChatCoreError::Unauthorized(
                        "Only owner can delete chat".to_string(),
                    ));
                }
            }
            None => return Err(ChatCoreError::NotFound("Chat not found".to_string())),
        }

        let chat = query_as(
            r#"
            DELETE FROM chats
            WHERE id = $1 AND $2 = ANY(members)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }

    pub async fn update_owner(
        id: i64,
        user_id: i64,
        new_owner_id: i64,
        pool: &PgPool,
    ) -> Result<Self, ChatCoreError> {
        let chat = Chat::find_chat_by_id(id, pool).await?;
        match chat {
            Some(chat) => {
                if chat.typ != ChatType::Single && chat.owner_id != Some(user_id) {
                    return Err(ChatCoreError::Unauthorized(
                        "Only owner can update chat".to_string(),
                    ));
                }
                if chat.typ == ChatType::Single {
                    return Ok(chat);
                }
            }
            None => return Err(ChatCoreError::NotFound("Chat not found".to_string())),
        }

        let chat = query_as(
            r#"
            UPDATE chats
            SET owner_id = $1
            WHERE id = $2 AND $1 = ANY(members) AND $3 = ANY(members)
            RETURNING *
            "#,
        )
        .bind(new_owner_id)
        .bind(id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        info!("Chat {} owner updated to {}", id, new_owner_id);
        Ok(chat)
    }

    pub async fn is_chat_member(
        id: i64,
        user_id: i64,
        pool: &PgPool,
    ) -> Result<bool, ChatCoreError> {
        let chat: Option<Chat> = query_as(
            r#"
            SELECT *
            FROM chats
            WHERE id = $1 AND $2 = ANY(members)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
        Ok(chat.is_some())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::get_test_pool;

    use super::*;

    #[tokio::test]
    async fn test_create_update_chat() -> anyhow::Result<()> {
        let (pool, _tdb) = get_test_pool(None).await;
        let name = "test_chat".to_string();
        let create_chat = CreateChat {
            name: Some(name.clone()),
            members: vec![1, 2, 3, 4],
            is_public: false,
        };
        let chat = Chat::create(create_chat, 1, 1, &pool).await?;
        assert_eq!(chat.name, Some(name));
        assert_eq!(chat.members, vec![1, 2, 3, 4]);
        assert_eq!(chat.typ, ChatType::Group);
        assert_eq!(chat.owner_id, Some(1));

        let chat = Chat::update_owner(chat.id, 1, 2, &pool).await?;
        assert_eq!(chat.owner_id, Some(2));

        Ok(())
    }

    #[tokio::test]
    async fn test_find_chat_by_id() -> anyhow::Result<()> {
        let (pool, _tdb) = get_test_pool(None).await;
        let chat = Chat::find_chat_by_id(1, &pool).await?;
        assert!(chat.is_some());
        let chat = chat.unwrap();
        assert_eq!(chat.typ, ChatType::Group);
        assert_eq!(chat.members, vec![1, 2, 3, 4]);
        assert_eq!(chat.name, Some("group_chat".to_string()));
        Ok(())
    }
}
