use std::convert::Infallible;

use axum::response::Sse;
use axum_extra::{headers, TypedHeader};
use futures::{SinkExt, Stream};
use tracing::Event;

pub(crate) async fn sse_handler(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut sse = Sse::new(());
    sse.send(Event::default().data(user_agent.to_string()))
        .unwrap();
    sse
}
