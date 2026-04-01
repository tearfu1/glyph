use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::errors::AppError;
use crate::models::{ReadingStatus, ReadingStatusType, SetReadingStatus};
use crate::services::reading_status as reading_status_service;
use crate::AppState;

pub async fn get_statuses() -> Json<Vec<ReadingStatusType>> {
    Json(reading_status_service::get_statuses().await)
}

pub async fn get_my_statuses(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<ReadingStatus>>, AppError> {
    let statuses = reading_status_service::get_my_statuses(&state.pool, auth.0.sub).await?;
    Ok(Json(statuses))
}

pub async fn set_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(book_id): Path<Uuid>,
    Json(input): Json<SetReadingStatus>,
) -> Result<Json<ReadingStatus>, AppError> {
    let record = reading_status_service::set_reading_status(
        &state.pool,
        auth.0.sub,
        book_id,
        input.status,
    )
    .await?;
    Ok(Json(record))
}
