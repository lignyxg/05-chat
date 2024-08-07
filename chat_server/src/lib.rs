use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use axum::middleware::from_fn_with_state;
use axum::response::IntoResponse;
use axum::routing::{get, patch, post};
use axum::Router;
use jwt_simple::prelude::ES256KeyPair;
use sqlx::PgPool;
use tracing::info;

use chat_core::middlewares::jwt::JwtVerify;
use chat_core::{middlewares::jwt::jwt_verify, utils::jwt::JwtSigner, User};
pub use config::AppConfig;
use handlers::*;

use crate::error::AppError;
use crate::middlewares::{verify_chat_member, with_middleware};

mod config;
mod error;
mod handlers;
mod middlewares;
pub mod models;

#[derive(Debug, Clone)]
pub struct ChatState {
    inner: Arc<ChatStateInner>,
}

pub struct ChatStateInner {
    pub(crate) config: AppConfig,
    pub(crate) pool: PgPool,
    pub(crate) jwt_signer: JwtSigner,
}

pub async fn get_router(state: ChatState) -> Router {
    let chat = Router::new()
        .route(
            "/:id",
            patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/:id/messages", get(list_messages_handler))
        .layer(from_fn_with_state(state.clone(), verify_chat_member))
        .route("/", get(list_chat_handler).post(create_chat_handler));

    let api = Router::new()
        .route(
            "/workspaces",
            get(list_workspace_handler).post(create_workspace_handler),
        )
        .route("/users", get(list_users_handler))
        .nest("/chat", chat)
        .route("/files", post(upload_file_handler))
        .route("/download/*url", get(download_file_handler))
        .layer(from_fn_with_state(state.clone(), jwt_verify::<ChatState>))
        .route("/signup", post(signup_handler))
        .route("/signin", post(signin_handler));

    let router = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);

    with_middleware(router)
}

impl JwtVerify for ChatState {
    type Error = AppError;
    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        self.jwt_signer.verify(token).map_err(AppError::from)
    }
}

impl Deref for ChatState {
    type Target = ChatStateInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Debug for ChatStateInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatState")
            .field("config", &self.config)
            .finish()
    }
}

impl ChatState {
    pub async fn new(config: AppConfig) -> Self {
        let pool = PgPool::connect(&config.db_url)
            .await
            .expect("Failed to connect to database");
        info!("connected to database: {}", config.db_url);
        let jwt_signer = JwtSigner::new(
            ES256KeyPair::from_pem(&config.auth.sk).expect("Failed to create jwt signer"),
        );
        Self {
            inner: Arc::new(ChatStateInner {
                config,
                pool,
                jwt_signer,
            }),
        }
    }
}

pub(crate) async fn index_handler() -> impl IntoResponse {
    "Hello, World!"
}

#[cfg(feature = "test-util")]
mod test_util {
    use sqlx::Executor;
    use sqlx_db_tester::TestPg;

    use super::*;

    impl ChatState {
        pub async fn new_for_test() -> (Self, TestPg) {
            let config = AppConfig::load().expect("Failed to load config");
            let server_url = config.db_url.rsplitn(2, '/').collect::<Vec<_>>();
            let server_url = server_url[1].to_string();
            eprintln!("server_url: {}", server_url);
            let tdb = TestPg::new(server_url, std::path::Path::new("../migrations"));
            let pool = tdb.get_pool().await;
            let jwt_signer = JwtSigner::new(
                ES256KeyPair::from_pem(&config.auth.sk).expect("Failed to create jwt signer"),
            );

            let state = Self {
                inner: Arc::new(ChatStateInner {
                    config,
                    pool,
                    jwt_signer,
                }),
            };
            (state, tdb)
        }
    }

    #[allow(unused)]
    pub async fn prepare_test_data(pool: &PgPool) {
        let sqls = include_str!("../fixtures/test.sql").split(';');
        let mut tx = pool.begin().await.expect("Failed to begin transaction");
        for sql in sqls {
            if sql.trim().is_empty() {
                continue;
            }
            tx.execute(sql).await.expect("Failed to execute sql");
        }
        tx.commit().await.expect("Failed to commit transaction");
    }
}
