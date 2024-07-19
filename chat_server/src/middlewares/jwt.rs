use axum::extract::{FromRequestParts, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use tracing::warn;

use crate::ChatState;

/*
    two ways to write middleware:

    1. use axum::middleware::from_fn or from_fn_with_state
    2. use tower::Service and tower::layer
*/

pub(crate) async fn jwt_verify(
    State(state): State<ChatState>,
    req: Request,
    next: Next,
) -> Response {
    // get token from request, if none, return 401
    let (mut parts, body) = req.into_parts();
    let token =
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
            Ok(TypedHeader(Authorization(bearer))) => bearer.token().to_string(),
            Err(e) => {
                warn!("error: {}", e);
                return (StatusCode::BAD_REQUEST, "bad request").into_response();
            }
        };

    let mut req = Request::from_parts(parts, body);
    match state.jwt_signer.verify(&token) {
        Ok(user) => {
            req.extensions_mut().insert(user);
            next.run(req).await
        }
        Err(_) => (StatusCode::UNAUTHORIZED, "verify token failed").into_response(),
    }
}
