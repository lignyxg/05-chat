use serde::{Deserialize, Serialize};

mod chat_file;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateChat {
    pub new_owner_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatFile {
    pub ext: Option<String>,
    pub ws_id: i64,
    pub hash: String,
}
