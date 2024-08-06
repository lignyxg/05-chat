use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use jwt_simple::reexports::thiserror::Error;

#[derive(Error, Debug)]
pub enum ChatCoreError {
    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("password hash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("{0} not found")]
    NotFound(String),
    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),
    #[error("email already exists: {0}")]
    EmailAlreadyExists(String),
    #[error("create chat error: {0}")]
    CreateChatError(String),
}

impl IntoResponse for ChatCoreError {
    fn into_response(self) -> Response {
        let status = match self {
            ChatCoreError::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ChatCoreError::PasswordHashError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ChatCoreError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ChatCoreError::NotFound(_) => StatusCode::NOT_FOUND,
            ChatCoreError::JwtError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ChatCoreError::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            ChatCoreError::CreateChatError(_) => StatusCode::BAD_REQUEST,
        };

        (status, Json(self.to_string())).into_response()
    }
}
