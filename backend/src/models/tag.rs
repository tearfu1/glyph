use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "tag_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TagType {
    Genre,
    Mood,
    Theme,
    Period,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub tag_type: TagType,
}

#[derive(Debug, Serialize)]
pub struct GroupedTags {
    pub tag_type: TagType,
    pub tags: Vec<Tag>,
}
