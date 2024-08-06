use sqlx::{query_as, PgPool};

use crate::error::ChatCoreError;
use crate::models::{CreateMessage, ListMessages, Messages};

impl Messages {
    pub async fn create(
        create_message: CreateMessage,
        sender_id: i64,
        chat_id: i64,
        pool: &PgPool,
    ) -> Result<Self, ChatCoreError> {
        let message = sqlx::query_as(
            r#"
            INSERT INTO messages
            (content, file, sender_id, chat_id)
            VALUES
            ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(create_message.content)
        .bind(create_message.file)
        .bind(sender_id)
        .bind(chat_id)
        .fetch_one(pool)
        .await?;

        Ok(message)
    }

    #[allow(unused)]
    pub async fn list_messages_in_chat(
        list_messages: ListMessages,
        chat_id: i64,
        pool: &PgPool,
    ) -> Result<Vec<Self>, ChatCoreError> {
        let last_id = list_messages.last_id.unwrap_or(i64::MAX);
        let messages: Vec<Messages> = query_as(
            r#"
            SELECT *
            FROM messages
            WHERE chat_id = $1 and id < $2
            ORDER BY created_at DESC
            LIMIT $3
            "#,
        )
        .bind(chat_id)
        .bind(last_id)
        .bind(list_messages.limit)
        .fetch_all(pool)
        .await?;
        Ok(messages)
    }
}
