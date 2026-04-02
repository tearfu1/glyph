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
use crate::models::{CreateQuestion, Question, QuestionWithUser, UserRole};
use crate::services::question as question_service;
use crate::AppState;

const DEFAULT_PAGE: i64 = 1;

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ReactionBody {
    pub is_like: bool,
}

pub async fn get_questions(
    State(state): State<AppState>,
    Path(book_id): Path<Uuid>,
    Query(query): Query<PageQuery>,
    auth: OptionalAuthUser,
) -> Result<Json<PaginatedResponse<QuestionWithUser>>, AppError> {
    let page = query.page.unwrap_or(DEFAULT_PAGE);
    let current_user_id = auth.0.map(|c| c.sub);
    let (data, total) = question_service::get_questions(&state.pool, book_id, page, current_user_id).await?;

    Ok(Json(PaginatedResponse {
        data,
        total,
        page,
        per_page: 20,
    }))
}

pub async fn get_my_questions_for_book(
    State(state): State<AppState>,
    Path(book_id): Path<Uuid>,
    auth: AuthUser,
) -> Result<Json<Vec<QuestionWithUser>>, AppError> {
    let data =
        question_service::get_my_questions_for_book(&state.pool, book_id, auth.0.sub).await?;
    Ok(Json(data))
}

pub async fn get_best_questions(
    State(state): State<AppState>,
    Path(book_id): Path<Uuid>,
    auth: OptionalAuthUser,
) -> Result<Json<Vec<QuestionWithUser>>, AppError> {
    let current_user_id = auth.0.map(|c| c.sub);
    let data = question_service::get_best_questions(&state.pool, book_id, current_user_id).await?;
    Ok(Json(data))
}

pub async fn create_question(
    State(state): State<AppState>,
    Path(book_id): Path<Uuid>,
    auth: AuthUser,
    Json(input): Json<CreateQuestion>,
) -> Result<(StatusCode, Json<Question>), AppError> {
    if auth.0.role != UserRole::Premium && auth.0.role != UserRole::Admin {
        return Err(AppError::Forbidden);
    }

    input
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let question =
        question_service::create_question(&state.pool, book_id, auth.0.sub, input).await?;
    Ok((StatusCode::CREATED, Json(question)))
}

pub async fn update_question(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
    Json(input): Json<CreateQuestion>,
) -> Result<Json<Question>, AppError> {
    input
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let question = question_service::update_question(&state.pool, id, auth.0.sub, input).await?;
    Ok(Json(question))
}

pub async fn delete_question(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<StatusCode, AppError> {
    question_service::delete_question(&state.pool, id, auth.0.sub).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_reaction(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
    Json(body): Json<ReactionBody>,
) -> Result<StatusCode, AppError> {
    question_service::add_question_reaction(&state.pool, id, auth.0.sub, body.is_like).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_reaction(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<StatusCode, AppError> {
    question_service::remove_question_reaction(&state.pool, id, auth.0.sub).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_incoming_questions(
    State(state): State<AppState>,
    Query(query): Query<PageQuery>,
    auth: AuthUser,
) -> Result<Json<PaginatedResponse<QuestionWithUser>>, AppError> {
    if auth.0.role != UserRole::Author && auth.0.role != UserRole::Admin {
        return Err(AppError::Forbidden);
    }

    let page = query.page.unwrap_or(DEFAULT_PAGE);
    let (data, total) =
        question_service::get_incoming_questions(&state.pool, auth.0.sub, page).await?;

    Ok(Json(PaginatedResponse {
        data,
        total,
        page,
        per_page: 20,
    }))
}

pub async fn get_answered_questions(
    State(state): State<AppState>,
    Query(query): Query<PageQuery>,
    auth: AuthUser,
) -> Result<Json<PaginatedResponse<QuestionWithUser>>, AppError> {
    if auth.0.role != UserRole::Author && auth.0.role != UserRole::Admin {
        return Err(AppError::Forbidden);
    }

    let page = query.page.unwrap_or(DEFAULT_PAGE);
    let (data, total) =
        question_service::get_answered_questions(&state.pool, auth.0.sub, page).await?;

    Ok(Json(PaginatedResponse {
        data,
        total,
        page,
        per_page: 20,
    }))
}

pub async fn get_my_questions(
    State(state): State<AppState>,
    Query(query): Query<PageQuery>,
    auth: AuthUser,
) -> Result<Json<PaginatedResponse<QuestionWithUser>>, AppError> {
    let page = query.page.unwrap_or(DEFAULT_PAGE);
    let (data, total) = question_service::get_my_questions(&state.pool, auth.0.sub, page).await?;

    Ok(Json(PaginatedResponse {
        data,
        total,
        page,
        per_page: 20,
    }))
}
