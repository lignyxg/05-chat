use axum::extract::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::error::AppError;
use crate::error::AppError::Unauthorized;

/*
    two ways to write middleware:

    1. use axum::middleware::from_fn or from_fn_with_state
    2. use tower::Service and tower::layer
*/

#[allow(unused)]
pub(crate) async fn mid_verify(req: Request, _next: Next) -> Result<impl IntoResponse, AppError> {
    let token = req
        .headers()
        .get("x-auth-token")
        .and_then(|it| it.to_str().ok());

    if token.is_none() || token.unwrap().is_empty() {
        return Err::<Response, AppError>(Unauthorized("token not found".to_string()));
    }
    todo!()
}
