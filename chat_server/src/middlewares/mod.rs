use axum::middleware::from_fn;
use axum::Router;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::Level;

use chat_core::middlewares::{request_id::with_request_id, server_time::ServerTimeLayer};
pub use chat_member::verify_chat_member;

mod chat_member;
pub(crate) fn with_middleware(router: Router) -> Router {
    router.layer(
        ServiceBuilder::new()
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
            .layer(CompressionLayer::new().gzip(true).br(true).deflate(true))
            .layer(from_fn(with_request_id))
            .layer(ServerTimeLayer),
    )
}
