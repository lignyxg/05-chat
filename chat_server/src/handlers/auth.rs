use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use chat_core::models::{CreateUser, SigninUser, User};

use crate::error::AppError;
use crate::ChatState;

pub(crate) async fn signin_handler(
    State(state): State<ChatState>,
    Json(SigninUser { email, password }): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    match User::verify_password(&email, &password, &state.pool).await {
        Ok(user) => {
            let token = state.jwt_signer.sign(user)?;
            info!("user {} signed in", email);
            Ok((StatusCode::OK, Json(AuthToken { token })).into_response())
        }
        Err(err) => {
            warn!("error: {}", err);
            Err(AppError::from(err))
        }
    }
}

pub(crate) async fn signup_handler(
    State(state): State<ChatState>,
    Json(create_user): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let email = &create_user.email.clone();
    let user = User::create(create_user, &state.pool).await?;
    let token = state.jwt_signer.sign(user)?;
    info!("user {} signed up", email);
    Ok((StatusCode::CREATED, Json(AuthToken { token })))
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AuthToken {
    token: String,
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use http_body_util::BodyExt;

    use chat_core::utils::jwt::JwtSigner;

    use crate::AppConfig;

    use super::*;

    #[tokio::test]
    async fn test_jwt_sign_verify() {
        let signer = JwtSigner::load("./fixtures/pkcs8.pem").expect("Failed to load ek.pem");
        let user = User {
            id: 1,
            ws_id: 0,
            fullname: "lign".to_string(),
            email: "testlign@gmail.com".to_string(),
            password_hash: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let token = signer.sign(user.clone()).unwrap();
        eprintln!("token: {}", token);

        let res = signer.verify(&token).unwrap();
        assert_eq!(res, user);
    }

    #[tokio::test]
    async fn test_signup_handler() -> Result<()> {
        let (state, _tdb) = ChatState::new_for_test(AppConfig::load().unwrap()).await;
        let create_user = CreateUser {
            ws_name: "test_ws".to_string(),
            fullname: "lign".to_string(),
            email: "testlign@gmail.com".to_string(),
            password: "password123".to_string(),
        };
        let res = signup_handler(State(state), Json(create_user))
            .await?
            .into_response();
        assert_eq!(res.status(), StatusCode::CREATED);
        let body = res.into_body().collect().await?.to_bytes();
        let body = serde_json::from_slice::<AuthToken>(&body)?;
        assert_ne!(body.token, "");
        Ok(())
    }

    #[tokio::test]
    async fn test_signin_handler() {
        let (state, _tdb) = ChatState::new_for_test(AppConfig::load().unwrap()).await;
        let create_user = CreateUser {
            ws_name: "test_ws".to_string(),
            fullname: "lign".to_string(),
            email: "testlign@gmail.com".to_string(),
            password: "password123".to_string(),
        };
        User::create(create_user, &state.pool).await.unwrap();
        let signin_user = SigninUser {
            email: "testlign@gmail.com".to_string(),
            password: "password123".to_string(),
        };
        let res = signin_handler(State(state), Json(signin_user))
            .await
            .into_response();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let body = serde_json::from_slice::<AuthToken>(&body).unwrap();
        assert_ne!(body.token, "");
    }
}
