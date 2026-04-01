use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::AuthUser;
use crate::errors::AppError;
use crate::handlers::PaginatedResponse;
use crate::models::{Book, BookQuery, BookWithAuthor, CreateBook, ReadingStatusType, UserRole};
use crate::services::book as book_service;
use crate::AppState;

const DEFAULT_PAGE: i64 = 1;

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ShelfQuery {
    pub page: Option<i64>,
    pub status: Option<ReadingStatusType>,
}

pub async fn get_books(
    State(state): State<AppState>,
    Query(query): Query<BookQuery>,
) -> Result<Json<PaginatedResponse<BookWithAuthor>>, AppError> {
    let page = query.page.unwrap_or(DEFAULT_PAGE);
    let (data, total) =
        book_service::get_books(&state.pool, page, query.search, query.tags).await?;

    Ok(Json(PaginatedResponse {
        data,
        total,
        page,
        per_page: 20,
    }))
}

pub async fn get_book(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BookWithAuthor>, AppError> {
    let book = book_service::get_book_by_id(&state.pool, id).await?;
    Ok(Json(book))
}

pub async fn add_book(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<CreateBook>,
) -> Result<(StatusCode, Json<Book>), AppError> {
    if auth.0.role != UserRole::Author && auth.0.role != UserRole::Admin {
        return Err(AppError::Forbidden);
    }

    input
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let book = book_service::create_book(&state.pool, auth.0.sub, input).await?;
    Ok((StatusCode::CREATED, Json(book)))
}

pub async fn get_shelf(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<ShelfQuery>,
) -> Result<Json<PaginatedResponse<Book>>, AppError> {
    let page = query.page.unwrap_or(DEFAULT_PAGE);
    let (data, total) =
        book_service::get_books_for_shelf(&state.pool, user_id, query.status, page).await?;

    Ok(Json(PaginatedResponse {
        data,
        total,
        page,
        per_page: 20,
    }))
}

pub async fn get_by_author(
    State(state): State<AppState>,
    Path(author_id): Path<Uuid>,
    Query(query): Query<PageQuery>,
) -> Result<Json<PaginatedResponse<Book>>, AppError> {
    let page = query.page.unwrap_or(DEFAULT_PAGE);
    let (data, total) = book_service::get_books_by_author(&state.pool, author_id, page).await?;

    Ok(Json(PaginatedResponse {
        data,
        total,
        page,
        per_page: 20,
    }))
}
