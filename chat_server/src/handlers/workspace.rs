use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};

use chat_core::models::{CreateWorkspace, User, Workspace};

use crate::error::AppError;
use crate::ChatState;

#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = 200, description = "List users in the workspace", body = [User])
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn list_users_handler(
    State(state): State<ChatState>,
    Extension(user): Extension<User>,
) -> Result<impl IntoResponse, AppError> {
    let users = User::list_users_by_workspace(user.ws_id, &state.pool).await?;

    Ok(Json(users))
}

#[utoipa::path(
    get,
    path = "/api/workspaces",
    responses(
        (status = 200, description = "List all workspaces", body = [Workspace])
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn list_workspace_handler(
    State(state): State<ChatState>,
) -> Result<impl IntoResponse, AppError> {
    let ws = Workspace::list_workspaces(&state.pool).await?;

    Ok(Json(ws))
}

#[utoipa::path(
    post,
    path = "/api/workspaces",
    responses(
        (status = 200, description = "Create a new workspace", body = Workspace)
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn create_workspace_handler(
    State(state): State<ChatState>,
    Json(create_workspace): Json<CreateWorkspace>,
) -> Result<impl IntoResponse, AppError> {
    let ws = Workspace::create(create_workspace, &state.pool).await?;
    let ws = serde_json::to_string(&ws)?;

    Ok((StatusCode::OK, ws))
}
