use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use tracing::warn;

use uuid::Uuid;

const X_REQUEST_ID: &str = "x-request-id";

pub async fn with_request_id(mut req: Request, next: Next) -> Response {
    let id = match req.headers().get(X_REQUEST_ID) {
        Some(v) => Some(v.clone()),
        None => {
            let uuid = Uuid::now_v7().to_string();
            match HeaderValue::from_str(&uuid) {
                Ok(uuid) => {
                    req.headers_mut().insert(X_REQUEST_ID, uuid.clone());
                    Some(uuid)
                }
                Err(e) => {
                    warn!("Failed to parse uuid: {}", e);
                    None
                }
            }
        }
    };

    let mut res = next.run(req).await;
    if let Some(id) = id {
        res.headers_mut().insert(X_REQUEST_ID, id);
    }
    res
}
