use axum::response::IntoResponse;

pub(crate) async fn list_chat_handler() -> impl IntoResponse {
    "list chat handler"
}

pub(crate) async fn create_chat_handler() -> impl IntoResponse {
    "create chat handler"
}

pub(crate) async fn delete_chat_handler() -> impl IntoResponse {
    "delete chat handler"
}

pub(crate) async fn update_chat_handler() -> impl IntoResponse {
    "update chat handler"
}
