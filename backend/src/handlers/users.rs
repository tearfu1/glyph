use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::AuthUser;
use crate::errors::AppError;
use crate::handlers::PaginatedResponse;
use crate::models::{PublicUser, ReviewWithUser, UpdateUser, UpdateUserRole, UserNavigation, UserRole};
use crate::services::user as user_service;
use crate::AppState;

const DEFAULT_PAGE: i64 = 1;

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
}

pub async fn get_user_profile(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<PublicUser>, AppError> {
    let user = user_service::get_user_profile(&state.pool, user_id).await?;
    Ok(Json(user))
}

pub async fn get_user_settings(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<PublicUser>, AppError> {
    let user = user_service::get_user_settings(&state.pool, auth.0.sub).await?;
    Ok(Json(user.into()))
}

pub async fn update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<UpdateUser>,
) -> Result<Json<PublicUser>, AppError> {
    input
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let user = user_service::update_user(&state.pool, auth.0.sub, input).await?;
    Ok(Json(user))
}

pub async fn get_user_navigation(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<UserNavigation>, AppError> {
    let nav = user_service::get_user_navigation(&state.pool, auth.0.sub).await?;
    Ok(Json(nav))
}

pub async fn get_user_reviews(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<PageQuery>,
) -> Result<Json<PaginatedResponse<ReviewWithUser>>, AppError> {
    let page = query.page.unwrap_or(DEFAULT_PAGE);
    let (data, total) = user_service::get_user_reviews(&state.pool, user_id, page).await?;
    Ok(Json(PaginatedResponse {
        data,
        total,
        page,
        per_page: 20,
    }))
}

pub async fn get_all_groups(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<UserRole>>, AppError> {
    if auth.0.role != UserRole::Admin {
        return Err(AppError::Forbidden);
    }
    let groups = user_service::get_all_groups(&state.pool).await;
    Ok(Json(groups))
}

pub async fn update_user_groups(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(user_id): Path<Uuid>,
    Json(input): Json<UpdateUserRole>,
) -> Result<Json<PublicUser>, AppError> {
    if auth.0.role != UserRole::Admin {
        return Err(AppError::Forbidden);
    }
    let user = user_service::update_user_groups(&state.pool, user_id, input.role).await?;
    Ok(Json(user))
}
