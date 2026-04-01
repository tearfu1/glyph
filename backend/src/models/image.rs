use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Image {
    pub id: Uuid,
    pub user_id: Uuid,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub original_name: Option<String>,
    pub size_bytes: Option<i32>,
    pub created_at: DateTime<Utc>,
}
