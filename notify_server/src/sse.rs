use axum::response::sse::{Event, KeepAlive};
use axum::response::Sse;
use axum_extra::{headers, TypedHeader};
use futures::Stream;
use futures_util::stream;
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::StreamExt;

pub(crate) async fn sse_handler(
    TypedHeader(_user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::repeat_with(|| Event::default().data("hi!"))
        .map(Ok)
        .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(KeepAlive::default())
}
