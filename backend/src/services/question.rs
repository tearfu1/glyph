use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{CreateQuestion, Question, QuestionWithUser};

const PAGE_SIZE: i64 = 20;

/// Build the common SELECT for questions with user info.
/// `user_id_param` — SQL bind placeholder for current user, e.g. `Some("$4")`.
/// When `None`, `user_reaction` is always NULL.
fn question_select(user_id_param: Option<&str>) -> String {
    let user_reaction = match user_id_param {
        Some(p) => format!(
            "(SELECT is_like FROM up_question_reaction WHERE question_id = q.id AND user_id = {}) AS user_reaction",
            p
        ),
        None => "NULL::boolean AS user_reaction".to_string(),
    };

    format!(
        r#"SELECT q.id, q.book_id, q.user_id, q.text, q.created_at, q.updated_at,
        u.login AS user_login,
        u.display_name AS user_display_name,
        u.avatar_url AS user_avatar_url,
        COALESCE((SELECT COUNT(*) FROM up_question_reaction WHERE question_id = q.id AND is_like = true), 0) AS like_count,
        COALESCE((SELECT COUNT(*) FROM up_question_reaction WHERE question_id = q.id AND is_like = false), 0) AS dislike_count,
        {user_reaction},
        EXISTS(SELECT 1 FROM up_answer WHERE question_id = q.id) AS has_answer,
        a.text AS answer_text,
        a.created_at AS answer_created_at,
        au.display_name AS answer_user_display_name,
        au.avatar_url AS answer_user_avatar_url
    FROM up_question q
    JOIN up_user u ON u.id = q.user_id
    LEFT JOIN up_answer a ON a.question_id = q.id
    LEFT JOIN up_user au ON au.id = a.user_id"#
    )
}

pub async fn get_questions(
    pool: &PgPool,
    book_id: Uuid,
    page: i64,
    per_page: i64,
    current_user_id: Option<Uuid>,
) -> Result<(Vec<QuestionWithUser>, i64), AppError> {
    let offset = (page.max(1) - 1) * per_page;

    let rows = match current_user_id {
        Some(uid) => {
            sqlx::query_as::<_, QuestionWithUser>(&format!(
                "{} WHERE q.book_id = $1 ORDER BY q.created_at DESC LIMIT $2 OFFSET $3",
                question_select(Some("$4"))
            ))
            .bind(book_id)
            .bind(per_page)
            .bind(offset)
            .bind(uid)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, QuestionWithUser>(&format!(
                "{} WHERE q.book_id = $1 ORDER BY q.created_at DESC LIMIT $2 OFFSET $3",
                question_select(None)
            ))
            .bind(book_id)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
    };

    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM up_question WHERE book_id = $1",
    )
    .bind(book_id)
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

pub async fn get_my_questions_for_book(
    pool: &PgPool,
    book_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<QuestionWithUser>, AppError> {
    let rows = sqlx::query_as::<_, QuestionWithUser>(&format!(
        "{} WHERE q.book_id = $1 AND q.user_id = $2 ORDER BY q.created_at DESC",
        question_select(Some("$2"))
    ))
    .bind(book_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn get_best_questions(
    pool: &PgPool,
    book_id: Uuid,
    current_user_id: Option<Uuid>,
) -> Result<Vec<QuestionWithUser>, AppError> {
    let rows = match current_user_id {
        Some(uid) => {
            sqlx::query_as::<_, QuestionWithUser>(&format!(
                r#"{} WHERE q.book_id = $1
                ORDER BY (
                    SELECT COUNT(*) FROM up_question_reaction WHERE question_id = q.id AND is_like = true
                ) DESC
                LIMIT 5"#,
                question_select(Some("$2"))
            ))
            .bind(book_id)
            .bind(uid)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, QuestionWithUser>(&format!(
                r#"{} WHERE q.book_id = $1
                ORDER BY (
                    SELECT COUNT(*) FROM up_question_reaction WHERE question_id = q.id AND is_like = true
                ) DESC
                LIMIT 5"#,
                question_select(None)
            ))
            .bind(book_id)
            .fetch_all(pool)
            .await?
        }
    };

    Ok(rows)
}

pub async fn create_question(
    pool: &PgPool,
    book_id: Uuid,
    user_id: Uuid,
    input: CreateQuestion,
) -> Result<Question, AppError> {
    let question = sqlx::query_as::<_, Question>(
        r#"
        INSERT INTO up_question (id, book_id, user_id, text)
        VALUES (gen_random_uuid(), $1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(book_id)
    .bind(user_id)
    .bind(&input.text)
    .fetch_one(pool)
    .await?;

    Ok(question)
}

pub async fn update_question(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
    input: CreateQuestion,
) -> Result<Question, AppError> {
    let question = sqlx::query_as::<_, Question>(
        "SELECT * FROM up_question WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Question {} not found", id)))?;

    if question.user_id != user_id {
        return Err(AppError::Forbidden);
    }

    let updated = sqlx::query_as::<_, Question>(
        r#"
        UPDATE up_question SET text = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING *
        "#,
    )
    .bind(&input.text)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(updated)
}

pub async fn delete_question(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let question = sqlx::query_as::<_, Question>(
        "SELECT * FROM up_question WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Question {} not found", id)))?;

    if question.user_id != user_id {
        return Err(AppError::Forbidden);
    }

    sqlx::query("DELETE FROM up_question WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn add_question_reaction(
    pool: &PgPool,
    question_id: Uuid,
    user_id: Uuid,
    is_like: bool,
) -> Result<(), AppError> {
    sqlx::query_as::<_, Question>(
        "SELECT * FROM up_question WHERE id = $1",
    )
    .bind(question_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Question {} not found", question_id)))?;

    sqlx::query(
        r#"
        INSERT INTO up_question_reaction (question_id, user_id, is_like)
        VALUES ($1, $2, $3)
        ON CONFLICT (question_id, user_id) DO UPDATE SET is_like = $3
        "#,
    )
    .bind(question_id)
    .bind(user_id)
    .bind(is_like)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn remove_question_reaction(
    pool: &PgPool,
    question_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "DELETE FROM up_question_reaction WHERE question_id = $1 AND user_id = $2",
    )
    .bind(question_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_incoming_questions(
    pool: &PgPool,
    author_id: Uuid,
    page: i64,
) -> Result<(Vec<QuestionWithUser>, i64), AppError> {
    let offset = (page.max(1) - 1) * PAGE_SIZE;

    let rows = sqlx::query_as::<_, QuestionWithUser>(&format!(
        r#"{} JOIN up_book b ON b.id = q.book_id
        WHERE b.author_id = $1
          AND NOT EXISTS (SELECT 1 FROM up_answer WHERE question_id = q.id)
        ORDER BY q.created_at DESC LIMIT $2 OFFSET $3"#,
        question_select(Some("$1"))
    ))
    .bind(author_id)
    .bind(PAGE_SIZE)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM up_question q
        JOIN up_book b ON b.id = q.book_id
        WHERE b.author_id = $1
          AND NOT EXISTS (SELECT 1 FROM up_answer WHERE question_id = q.id)
        "#,
    )
    .bind(author_id)
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

pub async fn get_answered_questions(
    pool: &PgPool,
    author_id: Uuid,
    page: i64,
) -> Result<(Vec<QuestionWithUser>, i64), AppError> {
    let offset = (page.max(1) - 1) * PAGE_SIZE;

    let rows = sqlx::query_as::<_, QuestionWithUser>(&format!(
        r#"{} JOIN up_book b ON b.id = q.book_id
        WHERE b.author_id = $1
          AND EXISTS (SELECT 1 FROM up_answer WHERE question_id = q.id)
        ORDER BY q.created_at DESC LIMIT $2 OFFSET $3"#,
        question_select(Some("$1"))
    ))
    .bind(author_id)
    .bind(PAGE_SIZE)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM up_question q
        JOIN up_book b ON b.id = q.book_id
        WHERE b.author_id = $1
          AND EXISTS (SELECT 1 FROM up_answer WHERE question_id = q.id)
        "#,
    )
    .bind(author_id)
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

pub async fn get_my_questions(
    pool: &PgPool,
    user_id: Uuid,
    page: i64,
) -> Result<(Vec<QuestionWithUser>, i64), AppError> {
    let offset = (page.max(1) - 1) * PAGE_SIZE;

    let rows = sqlx::query_as::<_, QuestionWithUser>(&format!(
        "{} WHERE q.user_id = $1 ORDER BY q.created_at DESC LIMIT $2 OFFSET $3",
        question_select(Some("$1"))
    ))
    .bind(user_id)
    .bind(PAGE_SIZE)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM up_question WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}
