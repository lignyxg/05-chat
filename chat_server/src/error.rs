use axum::extract::multipart::MultipartError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    CreateFileError(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("multipart error: {0}")]
    MultiPartError(#[from] MultipartError),
    #[error("invalid header value: {0}")]
    InvalidHeaderValue(#[from] axum::http::header::InvalidHeaderValue),
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("serde json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("{0}")]
    ChatCoreError(#[from] chat_core::error::ChatCoreError),
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::ChatCoreError(e) => e.into_response(),
            AppError::Forbidden(_) => (StatusCode::FORBIDDEN, self.to_string()).into_response(),
            AppError::SerdeJsonError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
            AppError::CreateFileError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
            AppError::MultiPartError(_) => {
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
            AppError::InvalidHeaderValue(_) => {
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
            AppError::ParseError(_) => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            AppError::IOError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
        }
    }
}
