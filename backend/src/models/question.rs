use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Question {
    pub id: Uuid,
    pub book_id: Uuid,
    pub user_id: Uuid,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateQuestion {
    #[validate(length(min = 1))]
    pub text: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct QuestionWithUser {
    pub id: Uuid,
    pub book_id: Uuid,
    pub user_id: Uuid,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_login: String,
    pub user_display_name: String,
    pub user_avatar_url: Option<String>,
    pub like_count: i64,
    pub dislike_count: i64,
    pub has_answer: bool,
    pub answer_text: Option<String>,
    pub answer_created_at: Option<DateTime<Utc>>,
    pub answer_user_display_name: Option<String>,
    pub answer_user_avatar_url: Option<String>,
}
