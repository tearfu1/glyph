use sqlx::PgPool;

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
