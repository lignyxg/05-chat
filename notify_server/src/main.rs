use tokio::net::TcpListener;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer as _;

use notify_server::config::AppConfig;
use notify_server::{get_router, NotifState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = AppConfig::load().expect("Failed to load config");
    let addr = format!("0.0.0.0:{}", config.server.port);

    let state = NotifState::new(config);

    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);
    let router = get_router(state).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
