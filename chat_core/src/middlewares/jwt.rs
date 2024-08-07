use std::fmt;

use axum::extract::{FromRequestParts, Query, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use serde::Deserialize;
use tracing::warn;

use crate::User;

/*
    two ways to write middleware:

    1. use axum::middleware::from_fn or from_fn_with_state
    2. use tower::Service and tower::layer
*/
#[derive(Debug, Deserialize)]
struct Params {
    access_token: String,
}

pub trait JwtVerify {
    type Error: fmt::Debug;
    fn verify(&self, token: &str) -> Result<User, Self::Error>;
}

pub async fn jwt_verify<T>(State(state): State<T>, req: Request, next: Next) -> Response
where
    T: JwtVerify + Send + Sync + 'static,
{
    // get token from request, if none, return 401
    let (mut parts, body) = req.into_parts();

    let token =
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
            Ok(TypedHeader(Authorization(bearer))) => bearer.token().to_string(),
            Err(e) => {
                if e.is_missing() {
                    let params = Query::<Params>::from_request_parts(&mut parts, &state).await;
                    match params {
                        Ok(Query(Params { access_token })) => access_token,
                        Err(e) => {
                            warn!("error: {}", e);
                            return (StatusCode::BAD_REQUEST, "bad request").into_response();
                        }
                    }
                } else {
                    warn!("error: {}", e);
                    return (StatusCode::BAD_REQUEST, "bad request").into_response();
                }
            }
        };

    let mut req = Request::from_parts(parts, body);
    match state.verify(&token) {
        Ok(user) => {
            req.extensions_mut().insert(user);
            next.run(req).await
        }
        Err(_) => (StatusCode::UNAUTHORIZED, "verify token failed").into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use std::sync::Arc;

    use axum::body::Body;
    use axum::middleware::from_fn_with_state;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    use crate::error::ChatCoreError;
    use crate::models::User;
    use crate::utils::jwt::JwtSigner;

    use super::*;

    #[derive(Clone)]
    struct AppState(Arc<JwtSigner>);

    impl Deref for AppState {
        type Target = JwtSigner;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl JwtVerify for AppState {
        type Error = ChatCoreError;
        fn verify(&self, token: &str) -> Result<User, Self::Error> {
            self.0.verify(token)
        }
    }

    async fn handler() -> impl IntoResponse {
        "handler"
    }

    #[tokio::test]
    async fn test_jwt_verify_middleware() -> anyhow::Result<()> {
        let signer = JwtSigner::load("./fixtures/pkcs8.pem").expect("Failed to load ek.pem");
        let state = AppState(Arc::new(signer));
        let app = Router::new()
            .route("/", get(handler))
            .layer(from_fn_with_state(state.clone(), jwt_verify::<AppState>));

        let user = User::new(1, 0, "lign".to_string(), "testlign@gmail.com".to_string());

        let token = state.sign(user)?;
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

        // good token in query string
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/?access_token={}", token))
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
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("Authorization", format!("Bearer {}", "bad token"))
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        // bad token in query string
        let res = app
            .oneshot(
                Request::builder()
                    .uri(format!("/?access_token={}", "bad_token"))
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
