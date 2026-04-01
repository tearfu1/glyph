use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "reading_status_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ReadingStatusType {
    WantToRead,
    Reading,
    Read,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReadingStatus {
    pub id: Uuid,
    pub user_id: Uuid,
    pub book_id: Uuid,
    pub status: ReadingStatusType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SetReadingStatus {
    pub status: ReadingStatusType,
}
