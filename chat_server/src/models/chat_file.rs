use std::path::Path;
use std::str::FromStr;

use sha1::{Digest, Sha1};
use tokio::fs;
use tracing::info;

use crate::error::AppError;
use crate::models::ChatFile;

impl ChatFile {
    pub async fn create(
        name: &str,
        content: &[u8],
        ws_id: i64,
        base_url: &str,
    ) -> Result<Self, AppError> {
        let mut hasher = Sha1::new();
        hasher.update(content);
        let hash = format!("{:x}", hasher.finalize());
        let ext = name.split('.').last().map(|s| s.to_string());

        let file = Self { ext, ws_id, hash };
        let url = file.local_path(base_url, ws_id);
        Self::upload(url, content).await?;

        Ok(file)
    }

    pub fn hash_to_path(&self, ws_id: i64) -> String {
        let (part1, part2) = self.hash.split_at(3);
        let (part2, part3) = part2.split_at(3);
        match self.ext.as_ref() {
            Some(ext) => format!("files/{}/{}/{}/{}.{}", ws_id, part1, part2, part3, ext),
            None => format!("files/{}/{}/{}/{}", ws_id, part1, part2, part3),
        }
    }

    pub fn local_path(&self, base_url: &str, ws_id: i64) -> String {
        format!("{}/{}", base_url, self.hash_to_path(ws_id))
    }

    pub fn exists(&self, base_url: &str, ws_id: i64) -> bool {
        Path::new(&self.local_path(base_url, ws_id)).exists()
    }

    pub async fn upload(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> Result<(), AppError> {
        let path = path.as_ref();
        if path.exists() {
            info!("file {} already exists", path.display());
            return Ok(());
        }
        fs::create_dir_all(&path.parent().expect("parent dir should exist")).await?;
        fs::write(path, content).await?;
        Ok(())
    }
}

impl FromStr for ChatFile {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path = s.strip_prefix("files/").expect("Invalid path");
        let (reaminder, ext) = match path.split_once('.') {
            Some((reaminder, ext)) => (reaminder, Some(ext.to_string())),
            None => (path, None),
        };
        let parts = reaminder.split('/').collect::<Vec<_>>();
        let ws_id = parts[0]
            .parse::<i64>()
            .map_err(|_e| AppError::CreateFileError("Invalid workspace id".to_string()))?;
        let hash = parts[1..].join("");
        Ok(Self { ext, ws_id, hash })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_file() -> anyhow::Result<()> {
        let name = "test.txt";
        let content = "Hello world!".as_bytes();
        let base_url = "/tmp/chat_server";
        let file = ChatFile::create(name, content, 1, base_url)
            .await
            .expect("Failed to create file");

        assert_eq!(file.ext, Some("txt".to_string()));
        assert_eq!(
            file.hash_to_path(1),
            "files/1/d34/86a/e9136e7856bc42212385ea797094475802.txt"
        );
        Ok(())
    }
}
