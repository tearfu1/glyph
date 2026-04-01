use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{Answer, CreateAnswer};

pub async fn create_answer(
    pool: &PgPool,
    question_id: Uuid,
    user_id: Uuid,
    input: CreateAnswer,
) -> Result<Answer, AppError> {
    // Check the question exists and get its book_id
    let book_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT book_id FROM up_question WHERE id = $1",
    )
    .bind(question_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Question {} not found", question_id)))?;

    // Check user is the author of the book
    let is_author = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM up_book WHERE id = $1 AND author_id = $2)",
    )
    .bind(book_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    if !is_author {
        return Err(AppError::Forbidden);
    }

    // Enforce one answer per question
    let already_answered = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM up_answer WHERE question_id = $1)",
    )
    .bind(question_id)
    .fetch_one(pool)
    .await?;

    if already_answered {
        return Err(AppError::Conflict("Question already has an answer".to_string()));
    }

    let answer = sqlx::query_as::<_, Answer>(
        r#"
        INSERT INTO up_answer (id, question_id, user_id, text)
        VALUES (gen_random_uuid(), $1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(question_id)
    .bind(user_id)
    .bind(&input.text)
    .fetch_one(pool)
    .await?;

    Ok(answer)
}
