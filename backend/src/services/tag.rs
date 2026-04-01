use std::collections::HashMap;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{GroupedTags, Tag, TagType};

pub async fn get_tags_grouped(
    pool: &PgPool,
    skip_types: Option<Vec<TagType>>,
) -> Result<Vec<GroupedTags>, AppError> {
    let all_tags = sqlx::query_as::<_, Tag>("SELECT id, name, tag_type FROM up_tag ORDER BY tag_type, name")
        .fetch_all(pool)
        .await?;

    let skip = skip_types.unwrap_or_default();

    let type_order = [TagType::Genre, TagType::Mood, TagType::Theme, TagType::Period];

    let mut result = Vec::new();
    for tag_type in &type_order {
        if skip.contains(tag_type) {
            continue;
        }
        let tags: Vec<Tag> = all_tags
            .iter()
            .filter(|t| &t.tag_type == tag_type)
            .cloned()
            .collect();
        if !tags.is_empty() {
            result.push(GroupedTags {
                tag_type: tag_type.clone(),
                tags,
            });
        }
    }

    Ok(result)
}

#[derive(sqlx::FromRow)]
struct BookTagRow {
    book_id: Uuid,
    id: Uuid,
    name: String,
    tag_type: TagType,
}

pub async fn get_tags_for_books(
    pool: &PgPool,
    book_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<Tag>>, AppError> {
    if book_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as::<_, BookTagRow>(
        r#"
        SELECT bt.book_id, t.id, t.name, t.tag_type
        FROM up_book_tag bt
        JOIN up_tag t ON t.id = bt.tag_id
        WHERE bt.book_id = ANY($1)
        ORDER BY t.tag_type, t.name
        "#,
    )
    .bind(book_ids)
    .fetch_all(pool)
    .await?;

    let mut map: HashMap<Uuid, Vec<Tag>> = HashMap::new();
    for row in rows {
        map.entry(row.book_id).or_default().push(Tag {
            id: row.id,
            name: row.name,
            tag_type: row.tag_type,
        });
    }

    Ok(map)
}
