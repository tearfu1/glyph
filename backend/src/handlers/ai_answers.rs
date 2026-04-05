use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::auth::middleware::{AuthUser, OptionalAuthUser};
use crate::errors::AppError;
use crate::models::{AiAnswerResponse, UserRole};
use crate::services::ai_answer as ai_answer_service;
use crate::AppState;

pub async fn generate_ai_answer(
    State(state): State<AppState>,
    Path(question_id): Path<Uuid>,
    auth: AuthUser,
) -> Result<(StatusCode, Json<AiAnswerResponse>), AppError> {
    if auth.0.role != UserRole::Author && auth.0.role != UserRole::Admin {
        return Err(AppError::Forbidden);
    }

    // Verify that the requesting user is the author of the book the question belongs to
    let is_book_author = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM up_question q
            JOIN up_book b ON b.id = q.book_id
            WHERE q.id = $1 AND b.author_id = $2
        )
        "#,
    )
    .bind(question_id)
    .bind(auth.0.sub)
    .fetch_one(&state.pool)
    .await?;

    if !is_book_author && auth.0.role != UserRole::Admin {
        return Err(AppError::Forbidden);
    }

    let ml_service_url = &state.config.ml_service_url;
    let ai_answer =
        ai_answer_service::generate_ai_answer(&state.pool, ml_service_url, question_id).await?;

    Ok((StatusCode::CREATED, Json(AiAnswerResponse::from(ai_answer))))
}

pub async fn get_ai_answer(
    State(state): State<AppState>,
    Path(question_id): Path<Uuid>,
    _auth: OptionalAuthUser,
) -> Result<Json<AiAnswerResponse>, AppError> {
    let ai_answer = ai_answer_service::get_ai_answer(&state.pool, question_id).await?;
    Ok(Json(AiAnswerResponse::from(ai_answer)))
}
