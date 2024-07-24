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

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::middleware::from_fn_with_state;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    use crate::models::User;
    use crate::AppConfig;

    use super::*;

    async fn handler() -> impl IntoResponse {
        "handler"
    }

    #[tokio::test]
    async fn test_jwt_verify_middleware() -> anyhow::Result<()> {
        let (state, _tdb) = ChatState::new_for_test(AppConfig::load()?).await;
        let app = Router::new()
            .route("/", get(handler))
            .layer(from_fn_with_state(state.clone(), jwt_verify));

        let user = User::new(1, 0, "lign".to_string(), "testlign@gmail.com".to_string());
        let token = state.jwt_signer.sign(user)?;
        // happy path
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(res.status(), StatusCode::OK);

        // bad request
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("Authorization", "Bad request")
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        // unauthorized
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("Authorization", format!("Bearer {}", "bad token"))
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
