use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotifyError {
    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),
    #[error("{0}")]
    NotificationFault(String),
}

impl IntoResponse for NotifyError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::JwtError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::NotificationFault(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(self.to_string())).into_response()
    }
}
