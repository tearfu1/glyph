use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::{AuthUser, OptionalAuthUser};
use crate::errors::AppError;
use crate::handlers::PaginatedResponse;
use crate::models::{CreateReview, Review, ReviewReaction, ReviewWithUser, UpdateReview};
use crate::services::review as review_service;
use crate::AppState;

const DEFAULT_PAGE: i64 = 1;

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ReactionRequest {
    pub is_like: bool,
}

pub async fn get_reviews(
    State(state): State<AppState>,
    Path((book_id,)): Path<(Uuid,)>,
    Query(query): Query<PageQuery>,
    auth: OptionalAuthUser,
) -> Result<Json<PaginatedResponse<ReviewWithUser>>, AppError> {
    let page = query.page.unwrap_or(DEFAULT_PAGE);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let current_user_id = auth.0.map(|c| c.sub);
    let (data, total) =
        review_service::get_reviews(&state.pool, book_id, page, per_page, current_user_id).await?;

    Ok(Json(PaginatedResponse {
        data,
        total,
        page,
        per_page,
    }))
}

pub async fn get_my_review(
    State(state): State<AppState>,
    Path((book_id,)): Path<(Uuid,)>,
    auth: AuthUser,
) -> Result<Json<Option<ReviewWithUser>>, AppError> {
    let review = review_service::get_my_review(&state.pool, book_id, auth.0.sub).await?;
    Ok(Json(review))
}

pub async fn create_review(
    State(state): State<AppState>,
    Path((book_id,)): Path<(Uuid,)>,
    auth: AuthUser,
    Json(input): Json<CreateReview>,
) -> Result<(StatusCode, Json<Review>), AppError> {
    input
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let review = review_service::create_review(&state.pool, book_id, auth.0.sub, input).await?;
    Ok((StatusCode::CREATED, Json(review)))
}

pub async fn update_review(
    State(state): State<AppState>,
    Path(review_id): Path<Uuid>,
    auth: AuthUser,
    Json(input): Json<UpdateReview>,
) -> Result<Json<Review>, AppError> {
    input
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let review =
        review_service::update_review(&state.pool, review_id, auth.0.sub, input).await?;
    Ok(Json(review))
}

pub async fn delete_review(
    State(state): State<AppState>,
    Path(review_id): Path<Uuid>,
    auth: AuthUser,
) -> Result<StatusCode, AppError> {
    review_service::delete_review(&state.pool, review_id, auth.0.sub).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_reaction(
    State(state): State<AppState>,
    Path(review_id): Path<Uuid>,
    auth: AuthUser,
    Json(input): Json<ReactionRequest>,
) -> Result<(StatusCode, Json<ReviewReaction>), AppError> {
    let reaction =
        review_service::add_reaction(&state.pool, review_id, auth.0.sub, input.is_like).await?;
    Ok((StatusCode::CREATED, Json(reaction)))
}

pub async fn remove_reaction(
    State(state): State<AppState>,
    Path(review_id): Path<Uuid>,
    auth: AuthUser,
) -> Result<StatusCode, AppError> {
    review_service::remove_reaction(&state.pool, review_id, auth.0.sub).await?;
    Ok(StatusCode::NO_CONTENT)
}
