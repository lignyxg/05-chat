use std::str::FromStr;

use axum::extract::{Multipart, Path, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use tokio::fs;
use tokio_util::io::ReaderStream;

use crate::error::AppError;
use crate::models::{ChatFile, User};
use crate::ChatState;

pub(crate) async fn upload_file_handler(
    State(state): State<ChatState>,
    Extension(user): Extension<User>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let base_url = state.config.base_url.clone();
    let ws_id = user.ws_id;

    let mut urls = Vec::new();

    while let Some(field) = multipart.next_field().await? {
        let name = field.file_name().unwrap().to_string();
        let content = field.bytes().await?;
        let chat_file = ChatFile::create(&name, content.as_ref(), ws_id, &base_url).await?;
        let url = chat_file.hash_to_path(ws_id);
        urls.push(url);
    }
    Ok(Json(urls))
}

pub(crate) async fn download_file_handler(
    State(state): State<ChatState>,
    Extension(user): Extension<User>,
    Path(url): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let chat_file = ChatFile::from_str(&url)?;
    if user.ws_id != chat_file.ws_id {
        return Err(AppError::Unauthorized(
            "you don't have permission".to_string(),
        ));
    }

    let url = chat_file.local_path(&state.config.base_url, user.ws_id);
    let file = fs::File::open(&url).await?;
    let stream = ReaderStream::new(file);

    let body = axum::body::Body::from_stream(stream);
    let mime = mime_guess::from_path(url).first_or_octet_stream();
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, mime.to_string().parse()?);

    Ok((headers, body))
}
