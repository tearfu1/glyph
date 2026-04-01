use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

use crate::errors::AppError;
use crate::models::{GroupedTags, TagType};
use crate::services::tag as tag_service;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct TagQuery {
    pub skip_types: Option<Vec<TagType>>,
}

pub async fn get_tags(
    State(state): State<AppState>,
    Query(query): Query<TagQuery>,
) -> Result<Json<Vec<GroupedTags>>, AppError> {
    let groups = tag_service::get_tags_grouped(&state.pool, query.skip_types).await?;
    Ok(Json(groups))
}
