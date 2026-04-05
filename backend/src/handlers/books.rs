use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::AuthUser;
use crate::errors::AppError;
use crate::handlers::PaginatedResponse;
use crate::models::{Book, BookQuery, BookWithAuthor, CreateBook, ReadingStatusType, Tag, UpdateBook, UserRole};
use crate::services::book as book_service;
use crate::services::tag as tag_service;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct BookWithTags {
    #[serde(flatten)]
    pub book: BookWithAuthor,
    pub tags: Vec<Tag>,
}

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
) -> Result<Json<PaginatedResponse<BookWithTags>>, AppError> {
    let page = query.page.unwrap_or(DEFAULT_PAGE);
    let tag_ids = query.tag_ids();
    let (books, total) =
        book_service::get_books(&state.pool, page, query.search, tag_ids).await?;

    let book_ids: Vec<Uuid> = books.iter().map(|b| b.id).collect();
    let mut tags_map = tag_service::get_tags_for_books(&state.pool, &book_ids).await?;

    let data = books
        .into_iter()
        .map(|book| {
            let tags = tags_map.remove(&book.id).unwrap_or_default();
            BookWithTags { book, tags }
        })
        .collect();

    Ok(Json(PaginatedResponse {
        data,
        total,
        page,
        per_page: 10,
    }))
}

pub async fn get_book(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BookWithTags>, AppError> {
    let book = book_service::get_book_by_id(&state.pool, id).await?;
    let tags = tag_service::get_tags_for_books(&state.pool, &[book.id])
        .await?
        .remove(&book.id)
        .unwrap_or_default();
    Ok(Json(BookWithTags { book, tags }))
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

pub async fn update_book(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateBook>,
) -> Result<Json<Book>, AppError> {
    let book = book_service::get_book_by_id(&state.pool, id).await?;

    if book.author_id != auth.0.sub && auth.0.role != UserRole::Admin {
        return Err(AppError::Forbidden);
    }

    input
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let updated = book_service::update_book(&state.pool, id, input).await?;
    Ok(Json(updated))
}

pub async fn get_shelf(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<ShelfQuery>,
) -> Result<Json<PaginatedResponse<BookWithTags>>, AppError> {
    let page = query.page.unwrap_or(DEFAULT_PAGE);
    let (books, total) =
        book_service::get_books_for_shelf(&state.pool, user_id, query.status, page).await?;

    let book_ids: Vec<Uuid> = books.iter().map(|b| b.id).collect();
    let mut tags_map = tag_service::get_tags_for_books(&state.pool, &book_ids).await?;

    let data = books
        .into_iter()
        .map(|book| {
            let tags = tags_map.remove(&book.id).unwrap_or_default();
            BookWithTags { book, tags }
        })
        .collect();

    Ok(Json(PaginatedResponse {
        data,
        total,
        page,
        per_page: 10,
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
        per_page: 10,
    }))
}
