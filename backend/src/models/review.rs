use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Review {
    pub id: Uuid,
    pub book_id: Uuid,
    pub user_id: Uuid,
    pub rating: i16,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateReview {
    #[validate(range(min = 1, max = 5))]
    pub rating: i16,
    #[validate(length(min = 1))]
    pub text: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateReview {
    #[validate(range(min = 1, max = 5))]
    pub rating: i16,
    #[validate(length(min = 1))]
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReviewReaction {
    pub id: Uuid,
    pub review_id: Uuid,
    pub user_id: Uuid,
    pub is_like: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ReviewWithUser {
    pub id: Uuid,
    pub book_id: Uuid,
    pub user_id: Uuid,
    pub rating: i16,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_login: String,
    pub user_display_name: String,
    pub user_avatar_url: Option<String>,
    pub like_count: i64,
    pub dislike_count: i64,
    pub user_reaction: Option<bool>,
}
