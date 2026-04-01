use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::AuthUser;
use crate::errors::AppError;
use crate::models::{Answer, CreateAnswer, UserRole};
use crate::services::answer as answer_service;
use crate::AppState;

pub async fn create_answer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
    Json(input): Json<CreateAnswer>,
) -> Result<(StatusCode, Json<Answer>), AppError> {
    if auth.0.role != UserRole::Author && auth.0.role != UserRole::Admin {
        return Err(AppError::Forbidden);
    }

    input
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let answer = answer_service::create_answer(&state.pool, id, auth.0.sub, input).await?;
    Ok((StatusCode::CREATED, Json(answer)))
}
