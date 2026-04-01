use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use crate::models::user::PublicUser;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Book {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub isbn: Option<String>,
    pub published_year: Option<i16>,
    pub author_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateBook {
    #[validate(length(min = 1, max = 512))]
    pub title: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub isbn: Option<String>,
    pub published_year: Option<i16>,
}

#[derive(Debug, Serialize)]
pub struct BookResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub isbn: Option<String>,
    pub published_year: Option<i16>,
    pub author_id: Uuid,
    pub author: PublicUser,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BookWithAuthor {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub isbn: Option<String>,
    pub published_year: Option<i16>,
    pub author_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Author fields (flattened from JOIN)
    pub author_login: String,
    pub author_email: String,
    pub author_display_name: String,
    pub author_avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BookQuery {
    pub page: Option<i64>,
    pub search: Option<String>,
    pub tags: Option<Vec<Uuid>>,
}
