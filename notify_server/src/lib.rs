use std::ops::Deref;
use std::sync::Arc;

use axum::middleware::from_fn_with_state;
use axum::routing::get;
use axum::Router;
use dashmap::DashMap;
use jwt_simple::prelude::ES256KeyPair;
use tokio::sync::broadcast::Sender;

use chat_core::middlewares::jwt::{jwt_verify, JwtVerify};
use chat_core::models::User;
use chat_core::utils::jwt::JwtSigner;

use crate::config::AppConfig;
use crate::notif::ChatEvent;
use crate::sse::sse_handler;

pub mod config;
mod error;
pub mod notif;
mod sse;

#[derive(Clone)]
pub struct NotifState {
    inner: Arc<NotifStateInner>,
}

pub struct NotifStateInner {
    pub config: AppConfig,
    verifier: JwtSigner,
    users_map: DashMap<i64, Sender<Arc<ChatEvent>>>,
}

pub fn get_router(state: NotifState) -> Router {
    Router::new()
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(state.clone(), jwt_verify::<NotifState>))
        .with_state(state)
}

impl JwtVerify for NotifState {
    type Error = chat_core::error::ChatCoreError;
    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        self.inner.verifier.verify(token)
    }
}

impl Deref for NotifState {
    type Target = NotifStateInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl NotifState {
    pub fn new() -> Self {
        let config = AppConfig::load().expect("Failed to load config");
        let verifier = JwtSigner::new(
            ES256KeyPair::from_pem(&config.auth.sk).expect("Failed to create jwt verifier"),
        );
        Self {
            inner: Arc::new(NotifStateInner {
                config,
                verifier,
                users_map: DashMap::new(),
            }),
        }
    }
}

impl Default for NotifState {
    fn default() -> Self {
        Self::new()
    }
}
