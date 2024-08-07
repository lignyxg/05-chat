use std::convert::Infallible;

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive};
use axum::response::Sse;
use axum::Extension;
use futures::Stream;
use futures_util::stream;
use jwt_simple::reexports::serde_json;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tracing::info;

use chat_core::User;

use crate::notif::ChatEvent;
use crate::NotifState;

const CHANNEL_CAP: usize = 256;

pub(crate) async fn sse_handler(
    Extension(user): Extension<User>,
    State(state): State<NotifState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let user_id = user.id;
    info!("user id: {}", user_id);
    let rx = if let Some(tx) = state.users_map.get(&user_id) {
        tx.subscribe()
    } else {
        let (tx, rx) = broadcast::channel(CHANNEL_CAP);
        state.users_map.insert(user_id, tx);
        rx
    };

    let stream = stream::unfold(
        rx,
        |mut rx| async move { Some((rx.recv().await.unwrap(), rx)) },
    )
    .map(|msg| {
        let name = match msg.as_ref() {
            ChatEvent::NewChat(_) => "new_chat",
            ChatEvent::UpdateChat(_) => "update_chat",
            ChatEvent::DeleteChat(_) => "delete_chat",
            ChatEvent::NewMessage(_) => "new_message",
        };
        let data = serde_json::to_string(&msg).expect("Failed to serialize data");
        Ok(Event::default().event(name).data(data))
    });
    info!("user {} subscribed", user.email);
    Sse::new(stream).keep_alive(KeepAlive::default())
}
