use std::net::SocketAddr;
use std::time::Duration;

use axum::http::StatusCode;
use futures::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use serde::Deserialize;
use serde_json::json;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

use chat_core::{Chat, CreateChat, SigninUser};

struct ChatServer {
    addr: SocketAddr,
    token: String,
    client: reqwest::Client,
}
#[derive(Debug, Deserialize)]
struct Token {
    token: String,
}

const WILD_ADDR: &str = "0.0.0.0:0";

struct NotifyServer;

#[tokio::test]
async fn test_chat_server() -> anyhow::Result<()> {
    // 1.create chat server
    let (state, tdb) = chat_server::ChatState::new_for_test().await;
    let mut chat_server = ChatServer::new(state).await?;
    // 2.sign in user
    let user = SigninUser {
        email: "alice@bbc.com".to_string(),
        password: "123456".to_string(),
    };
    let _token = chat_server.sign_in(user).await?;

    let db_url = tdb.url();
    NotifyServer::new(&db_url, &chat_server.token).await?;

    // 3.create chat
    let chat = chat_server
        .create_chat(CreateChat {
            name: Some("test_chat".to_string()),
            members: vec![1, 2, 3, 4],
            is_public: false,
        })
        .await?;
    // 4.send message
    chat_server.send_message(chat.id).await?;

    tokio::time::sleep(Duration::from_secs(1)).await;

    Ok(())
}

impl ChatServer {
    async fn new(state: chat_server::ChatState) -> anyhow::Result<Self> {
        let app = chat_server::get_router(state).await;
        let listener = TcpListener::bind(WILD_ADDR).await?;
        let addr = listener.local_addr()?;
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let client = reqwest::Client::new();

        Ok(Self {
            addr,
            token: "".to_string(),
            client,
        })
    }

    async fn sign_in(&mut self, user: SigninUser) -> anyhow::Result<String> {
        let res = self
            .client
            .post(format!("http://{}/api/signin", self.addr))
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&user)?)
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::OK);
        let token: Token = res.json().await?;
        self.token = token.token.clone();
        Ok(token.token)
    }

    async fn create_chat(&self, chat: CreateChat) -> anyhow::Result<Chat> {
        let res = self
            .client
            .post(format!("http://{}/api/chat", self.addr))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.token))
            .body(serde_json::to_string(&chat)?)
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::CREATED);
        let chat: Chat = res.json().await?;
        Ok(chat)
    }

    async fn send_message(&self, chat_id: i64) -> anyhow::Result<()> {
        let mut file = tokio::fs::File::open("../Cargo.toml").await?;
        let mut buf = Vec::new();
        file.read_buf(&mut buf).await?;

        let part = reqwest::multipart::Part::bytes(buf)
            .file_name("Cargo.toml")
            .mime_str("text/plain")?;
        let form = reqwest::multipart::Form::new().part("file", part);
        let res = self
            .client
            .post(format!("http://{}/api/files", self.addr))
            .header("Authorization", format!("Bearer {}", self.token))
            .multipart(form)
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::CREATED);

        let ret: Vec<String> = res.json().await?;

        let message_body = serde_json::to_string(&json!(
            {
                "content": "what's up",
                "file": ret,
            }
        ))?;

        let res = self
            .client
            .post(format!("http://{}/api/chat/{}", self.addr, chat_id))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.token))
            .body(message_body)
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::CREATED);

        Ok(())
    }
}

impl NotifyServer {
    async fn new(db_url: &str, token: &str) -> anyhow::Result<Self> {
        let mut config = notify_server::config::AppConfig::load()?;
        config.db_url = db_url.to_string();
        let state = notify_server::NotifState::new(config);

        let app = notify_server::get_router(state).await?;
        let listener = TcpListener::bind(WILD_ADDR).await?;
        let addr = listener.local_addr()?;

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let mut es = EventSource::get(format!("http://{}/events?access_token={}", addr, token));

        tokio::spawn(async move {
            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Open) => println!("connected"),
                    Ok(Event::Message(msg)) => println!("message: {:?}", msg),
                    Err(err) => {
                        println!("error: {}", err);
                        es.close();
                    }
                }
            }
        });

        Ok(Self)
    }
}
