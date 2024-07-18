use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use axum::response::IntoResponse;
use axum::routing::{get, patch, post};
use axum::Router;
use jwt_simple::prelude::ES256KeyPair;
use sqlx::PgPool;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::{info, Level};

pub use config::AppConfig;
use handlers::*;

use crate::utils::jwt::JwtSigner;

mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod utils;

#[derive(Debug, Clone)]
pub(crate) struct ChatState {
    inner: Arc<ChatStateInner>,
}

pub(crate) struct ChatStateInner {
    pub(crate) config: AppConfig,
    pub(crate) pool: sqlx::PgPool,
    pub(crate) jwt_signer: JwtSigner,
}

pub async fn get_router(config: AppConfig) -> Router {
    let state = ChatState::new(config).await;

    let api = Router::new()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Micros),
                ),
        )
        .route("/chat", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chat/:id",
            patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/chat/:id/messages", get(list_messages_handler))
        .route("/signup", post(signup_handler))
        .route("/signin", post(signin_handler));

    Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state)
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

#[cfg(test)]
impl ChatState {
    pub async fn new_for_test(config: AppConfig) -> (Self, sqlx_db_tester::TestPg) {
        use sqlx_db_tester::TestPg;

        let server_url = config.db_url.rsplitn(2, "/").collect::<Vec<_>>();
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
