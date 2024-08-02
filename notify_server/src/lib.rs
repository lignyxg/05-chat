use axum::routing::get;
use axum::Router;
use sqlx::postgres::PgListener;
use tracing::info;

use crate::sse::sse_handler;

pub mod config;
mod sse;

// pub enum ChatEvent {
//     NewChat(Chat),
//     UpdateChat(Chat),
//     NewMessage(Messages),
//     NewUser(User),
//     NewWorkspace(Workspace),
// }

pub fn get_router() -> Router {
    Router::new().route("/events", get(sse_handler))
}

pub async fn setup_pglistener(pg_url: &str) -> Result<(), sqlx::Error> {
    let mut listener = PgListener::connect(pg_url).await?;
    listener
        .listen_all(["chat_update", "messages_create"])
        .await?;

    tokio::spawn(async move {
        loop {
            match listener.recv().await {
                Ok(notification) => {
                    info!("Notification: {:?}", notification);
                }
                Err(err) => {
                    info!("error: {}", err);
                }
            }
            // if let Ok(notification) = listener.recv().await {
            //     info!("received notification");
            //     info!("Notification: {:?}", notification);
            // }
        }
    });

    Ok(())
}
