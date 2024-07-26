use axum::extract::{FromRequestParts, Path, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use tracing::warn;

use crate::error::AppError;
use crate::models::{Chat, User};
use crate::ChatState;

pub(crate) async fn verify_chat_member(
    State(state): State<ChatState>,
    req: Request,
    next: Next,
) -> Response {
    let (mut parts, body) = req.into_parts();
    let chat_id = match Path::<i64>::from_request_parts(&mut parts, &state).await {
        Ok(id) => id.0,
        Err(e) => {
            warn!("error: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                AppError::ParseError("Invalid chat id".to_string()),
            )
                .into_response();
        }
    };

    let user = match parts.extensions.get::<User>() {
        Some(user) => user.clone(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                AppError::Unauthorized("User not found".to_string()),
            )
                .into_response();
        }
    };
    let req = Request::from_parts(parts, body);

    match Chat::is_chat_member(chat_id, user.id, &state.pool).await {
        Ok(true) => next.run(req).await,
        Ok(false) => (
            StatusCode::FORBIDDEN,
            AppError::Unauthorized(format!("User {} is not in the chat", user.email)),
        )
            .into_response(),
        Err(e) => {
            warn!("error: {}", e);
            e.into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::StatusCode;
    use axum::middleware::from_fn_with_state;
    use axum::response::IntoResponse;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    use crate::middlewares::jwt::jwt_verify;
    use crate::test_util::prepare_test_data;
    use crate::AppConfig;

    use super::*;

    async fn handler() -> impl IntoResponse {
        (StatusCode::OK, "Okdokey")
    }

    #[tokio::test]
    async fn test_chat_member_middleware() -> anyhow::Result<()> {
        let (state, tdb) = ChatState::new_for_test(AppConfig::load()?).await;
        let pool = tdb.get_pool().await;
        prepare_test_data(&pool).await;

        let app = Router::new()
            .route("/:id", get(handler))
            .layer(from_fn_with_state(state.clone(), verify_chat_member))
            .layer(from_fn_with_state(state.clone(), jwt_verify));

        let user = User::find_user_by_email("alice@bbc.com", &pool)
            .await?
            .unwrap();
        let token = state.jwt_signer.sign(user)?;
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/1")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(res.status(), StatusCode::OK);

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/6")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(res.status(), StatusCode::FORBIDDEN);

        Ok(())
    }
}
