use std::collections::HashSet;
use std::sync::Arc;

use jwt_simple::reexports::serde_json;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use tracing::{info, warn};

use chat_core::{Chat, Messages};

use crate::error::NotifyError;
use crate::error::NotifyError::NotificationFault;
use crate::NotifState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChatEvent {
    NewChat(Chat),
    UpdateChat(Chat),
    DeleteChat(Chat),
    NewMessage(Messages),
}

pub async fn setup_pglistener(state: NotifState) -> Result<(), NotifyError> {
    let mut listener = PgListener::connect(&state.config.db_url).await?;
    listener
        .listen_all(["chat_update", "messages_create"])
        .await?;

    tokio::spawn(async move {
        loop {
            match listener.recv().await {
                Ok(notif) => {
                    info!("received notification: {:?}", notif);
                    let decoded = Notification::decode(notif.channel(), notif.payload())?;
                    for u in decoded.users {
                        if let Some(tx) = state.users_map.get(&u) {
                            if let Err(e) = tx.send(decoded.event.clone()) {
                                warn!("send chat event error: {}", e);
                            }
                        }
                    }
                }
                Err(err) => {
                    warn!("receive notification error: {}", err);
                }
            }
        }
        #[allow(unreachable_code)]
        Ok::<_, NotifyError>(())
    });

    Ok(())
}

pub struct Notification {
    pub event: Arc<ChatEvent>,
    pub users: HashSet<i64>,
}

impl Notification {
    pub fn decode(channel: &str, payload: &str) -> Result<Self, NotifyError> {
        match channel {
            "chat_update" => {
                let chat_update: ChatUpdate =
                    serde_json::from_str(payload).expect("Invalid chat update");

                // find users that need to be notified
                let users = Self::get_notified_users(&chat_update.old, &chat_update.new);

                let chat_event = match chat_update.op.as_str() {
                    "INSERT" => ChatEvent::NewChat(chat_update.new.expect("New chat should exist")),
                    "UPDATE" => {
                        ChatEvent::UpdateChat(chat_update.new.expect("New chat should exist"))
                    }
                    "DELETE" => {
                        ChatEvent::DeleteChat(chat_update.old.expect("Old chat should exist"))
                    }
                    _ => return Err(NotificationFault("unknown op".to_string())),
                };

                Ok(Self {
                    event: Arc::new(chat_event),
                    users,
                })
            }
            "messages_create" => {
                let message: MessageCreate =
                    serde_json::from_str(payload).expect("Invalid message");
                Ok(Self {
                    event: Arc::new(ChatEvent::NewMessage(message.messages)),
                    users: message.users.iter().copied().collect(),
                })
            }
            _ => {
                warn!("unknown channel: {}", channel);
                Err(NotificationFault("unknown channel".to_string()))
            }
        }
    }

    fn get_notified_users(old: &Option<Chat>, new: &Option<Chat>) -> HashSet<i64> {
        match (old, new) {
            (Some(old), Some(new)) => {
                let old_members = old.members.iter().copied().collect::<HashSet<_>>();
                let new_members = new.members.iter().copied().collect::<HashSet<_>>();
                if old_members == new_members {
                    HashSet::new()
                } else {
                    // inform the union of the old and new members
                    // if the old and new members are different
                    old_members.union(&new_members).copied().collect()
                }
            }
            (Some(old), None) => old.members.iter().copied().collect(),
            (None, Some(new)) => new.members.iter().copied().collect(),
            (None, None) => HashSet::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatUpdate {
    pub op: String,
    pub old: Option<Chat>,
    pub new: Option<Chat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageCreate {
    pub messages: Messages,
    pub users: Vec<i64>,
}
