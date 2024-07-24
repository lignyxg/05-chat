use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::{Extension, Json};

use crate::error::AppError;
use crate::models::{Chat, CreateChat, UpdateChat, User};
use crate::ChatState;

pub(crate) async fn list_chat_handler(
    State(state): State<ChatState>,
    Extension(user): Extension<User>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::list_chats_in_workspace(user.ws_id, &state.pool).await?;
    Ok(Json(chat))
}

pub(crate) async fn create_chat_handler(
    State(state): State<ChatState>,
    Extension(user): Extension<User>,
    Json(create_chat): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::create(create_chat, user.ws_id, user.id, &state.pool).await?;
    Ok(Json(chat))
}

pub(crate) async fn delete_chat_handler(
    State(state): State<ChatState>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::delete(id, user.id, &state.pool).await?;
    Ok(Json(chat))
}

pub(crate) async fn update_chat_handler(
    State(state): State<ChatState>,
    Extension(user): Extension<User>,
    Path(id): Path<i64>,
    Json(update_chat): Json<UpdateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::update_owner(id, user.id, update_chat.new_owner_id, &state.pool).await?;
    Ok(Json(chat))
}
