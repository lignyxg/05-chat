use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};

use crate::error::AppError;
use crate::models::{CreateWorkspace, User, Workspace};
use crate::ChatState;

pub(crate) async fn list_users_handler(
    State(state): State<ChatState>,
    Extension(user): Extension<User>,
) -> Result<impl IntoResponse, AppError> {
    let users = User::list_users_by_workspace(user.ws_id, &state.pool).await?;

    Ok(Json(users))
}

pub(crate) async fn list_workspace_handler(
    State(state): State<ChatState>,
) -> Result<impl IntoResponse, AppError> {
    let ws = Workspace::list_workspaces(&state.pool).await?;

    Ok(Json(ws))
}

pub(crate) async fn create_workspace_handler(
    State(state): State<ChatState>,
    Json(create_workspace): Json<CreateWorkspace>,
) -> Result<impl IntoResponse, AppError> {
    let ws = Workspace::create(create_workspace, &state.pool).await?;
    let ws = serde_json::to_string(&ws)?;

    Ok((StatusCode::OK, ws))
}
