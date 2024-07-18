use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
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
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("email already exists: {0}")]
    EmailAlreadyExists(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PasswordHashError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::JwtError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::IOError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::EmailAlreadyExists(_) => StatusCode::CONFLICT,
        };

        (status, Json(ErrorInfo::new(self.to_string()))).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub err: String,
}

impl ErrorInfo {
    pub fn new(err: impl Into<String>) -> Self {
        Self { err: err.into() }
    }
}
