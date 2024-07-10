use axum::response::IntoResponse;

pub(crate) async fn send_message_handler() -> impl IntoResponse {
    "send message handler"
}

pub(crate) async fn list_messages_handler() -> impl IntoResponse {
    "list messages handler"
}
