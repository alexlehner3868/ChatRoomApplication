use crate::message::ChatMessage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbUser {
    pub id: Uuid,
    pub user_id: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct DbRoom {
    pub id: Uuid,
    pub room_id: String,
    pub room_password_hash: String,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow, Debug)]
pub struct DbMessage {
    pub id: Uuid,
    pub room_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl From<DbMessage> for ChatMessage {
    fn from(m: DbMessage) -> Self {
        ChatMessage {
            room_id: m.room_id.to_string(),
            user_id: m.user_id.to_string(),
            message_id: m.id.to_string(),
            content: m.content,
            timestamp: m.created_at.to_rfc3339(),
        }
    }
}
