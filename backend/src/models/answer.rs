use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Answer {
    pub id: Uuid,
    pub question_id: Uuid,
    pub user_id: Uuid,
    pub text: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAnswer {
    #[validate(length(min = 1))]
    pub text: String,
}
