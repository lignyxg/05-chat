use std::mem;
use std::str::FromStr;

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use serde_json::json;
use tracing::warn;

use crate::error::AppError;
use crate::models::{ChatFile, CreateMessage, ListMessages, Messages, User};
use crate::ChatState;

pub(crate) async fn send_message_handler(
    State(state): State<ChatState>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
    Json(mut create_message): Json<CreateMessage>,
) -> Result<impl IntoResponse, AppError> {
    let mut non_exists_file = Vec::new();

    let chat_file = create_message
        .file
        .iter()
        .filter_map(|s| match ChatFile::from_str(s) {
            Ok(chat_file) => {
                if !chat_file.exists(&state.config.base_url, user.ws_id) {
                    warn!("file {} not found", s);
                    non_exists_file.push(s.clone());
                    None
                } else {
                    Some(s.clone())
                }
            }
            Err(_) => {
                warn!("Invalid file url {}", s);
                non_exists_file.push(s.clone());
                None
            }
        })
        .collect::<Vec<_>>();

    // for s in create_message.file.iter() {
    //     let chat_file = ChatFile::from_str(&s)?;
    //     if !chat_file.exists(&state.config.base_url, user.ws_id) {
    //         warn!("file {} not found", s);
    //         non_exists.push(s.clone());
    //     }
    // }

    let _ = mem::replace(&mut create_message.file, chat_file);

    let _message = Messages::create(create_message, user.id, id, &state.pool).await?;

    let ret = json!({"non_exists_file": non_exists_file});
    Ok(Json(ret))
}

pub(crate) async fn list_messages_handler(
    State(state): State<ChatState>,
    Path(id): Path<i64>,
    Query(list_messages): Query<ListMessages>,
) -> Result<impl IntoResponse, AppError> {
    let messages = Messages::list_messages_in_chat(list_messages, id, &state.pool).await?;
    Ok(Json(messages))
}
