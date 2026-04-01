use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{CreateReview, Review, ReviewReaction, ReviewWithUser, UpdateReview};

const PAGE_SIZE: i64 = 20;

pub async fn get_reviews(
    pool: &PgPool,
    book_id: Uuid,
    page: i64,
    current_user_id: Option<Uuid>,
) -> Result<(Vec<ReviewWithUser>, i64), AppError> {
    let offset = (page.max(1) - 1) * PAGE_SIZE;

    let rows = match current_user_id {
        Some(uid) => {
            sqlx::query_as::<_, ReviewWithUser>(
                r#"
                SELECT r.id, r.book_id, r.user_id, r.rating, r.text, r.created_at, r.updated_at,
                    u.login AS user_login,
                    u.display_name AS user_display_name,
                    u.avatar_url AS user_avatar_url,
                    COALESCE((SELECT COUNT(*) FROM up_review_reaction WHERE review_id = r.id AND is_like = true), 0) AS like_count,
                    COALESCE((SELECT COUNT(*) FROM up_review_reaction WHERE review_id = r.id AND is_like = false), 0) AS dislike_count,
                    (SELECT is_like FROM up_review_reaction WHERE review_id = r.id AND user_id = $3) AS user_reaction
                FROM up_review r
                JOIN up_user u ON u.id = r.user_id
                WHERE r.book_id = $1
                ORDER BY r.created_at DESC
                LIMIT $2 OFFSET $4
                "#,
            )
            .bind(book_id)
            .bind(PAGE_SIZE)
            .bind(uid)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, ReviewWithUser>(
                r#"
                SELECT r.id, r.book_id, r.user_id, r.rating, r.text, r.created_at, r.updated_at,
                    u.login AS user_login,
                    u.display_name AS user_display_name,
                    u.avatar_url AS user_avatar_url,
                    COALESCE((SELECT COUNT(*) FROM up_review_reaction WHERE review_id = r.id AND is_like = true), 0) AS like_count,
                    COALESCE((SELECT COUNT(*) FROM up_review_reaction WHERE review_id = r.id AND is_like = false), 0) AS dislike_count,
                    NULL::boolean AS user_reaction
                FROM up_review r
                JOIN up_user u ON u.id = r.user_id
                WHERE r.book_id = $1
                ORDER BY r.created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(book_id)
            .bind(PAGE_SIZE)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
    };

    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM up_review WHERE book_id = $1",
    )
    .bind(book_id)
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

pub async fn get_my_review(
    pool: &PgPool,
    book_id: Uuid,
    user_id: Uuid,
) -> Result<Option<ReviewWithUser>, AppError> {
    let row = sqlx::query_as::<_, ReviewWithUser>(
        r#"
        SELECT r.id, r.book_id, r.user_id, r.rating, r.text, r.created_at, r.updated_at,
            u.login AS user_login,
            u.display_name AS user_display_name,
            u.avatar_url AS user_avatar_url,
            COALESCE((SELECT COUNT(*) FROM up_review_reaction WHERE review_id = r.id AND is_like = true), 0) AS like_count,
            COALESCE((SELECT COUNT(*) FROM up_review_reaction WHERE review_id = r.id AND is_like = false), 0) AS dislike_count,
            (SELECT is_like FROM up_review_reaction WHERE review_id = r.id AND user_id = $2) AS user_reaction
        FROM up_review r
        JOIN up_user u ON u.id = r.user_id
        WHERE r.book_id = $1 AND r.user_id = $2
        "#,
    )
    .bind(book_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn create_review(
    pool: &PgPool,
    book_id: Uuid,
    user_id: Uuid,
    input: CreateReview,
) -> Result<Review, AppError> {
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM up_review WHERE book_id = $1 AND user_id = $2",
    )
    .bind(book_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    if existing > 0 {
        return Err(AppError::Conflict(
            "You have already reviewed this book".to_string(),
        ));
    }

    let review = sqlx::query_as::<_, Review>(
        r#"
        INSERT INTO up_review (id, book_id, user_id, rating, text)
        VALUES (gen_random_uuid(), $1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(book_id)
    .bind(user_id)
    .bind(input.rating)
    .bind(&input.text)
    .fetch_one(pool)
    .await?;

    Ok(review)
}

pub async fn update_review(
    pool: &PgPool,
    review_id: Uuid,
    user_id: Uuid,
    input: UpdateReview,
) -> Result<Review, AppError> {
    let review = sqlx::query_as::<_, Review>(
        r#"
        UPDATE up_review
        SET rating = $1, text = $2, updated_at = now()
        WHERE id = $3 AND user_id = $4
        RETURNING *
        "#,
    )
    .bind(input.rating)
    .bind(&input.text)
    .bind(review_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Review {} not found", review_id)))?;

    Ok(review)
}

pub async fn delete_review(
    pool: &PgPool,
    review_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let rows_affected = sqlx::query(
        "DELETE FROM up_review WHERE id = $1 AND user_id = $2",
    )
    .bind(review_id)
    .bind(user_id)
    .execute(pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(format!("Review {} not found", review_id)));
    }

    Ok(())
}

pub async fn add_reaction(
    pool: &PgPool,
    review_id: Uuid,
    user_id: Uuid,
    is_like: bool,
) -> Result<ReviewReaction, AppError> {
    let reaction = sqlx::query_as::<_, ReviewReaction>(
        r#"
        INSERT INTO up_review_reaction (id, review_id, user_id, is_like)
        VALUES (gen_random_uuid(), $1, $2, $3)
        ON CONFLICT (review_id, user_id)
        DO UPDATE SET is_like = EXCLUDED.is_like
        RETURNING *
        "#,
    )
    .bind(review_id)
    .bind(user_id)
    .bind(is_like)
    .fetch_one(pool)
    .await?;

    Ok(reaction)
}

pub async fn remove_reaction(
    pool: &PgPool,
    review_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let rows_affected = sqlx::query(
        "DELETE FROM up_review_reaction WHERE review_id = $1 AND user_id = $2",
    )
    .bind(review_id)
    .bind(user_id)
    .execute(pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound("Reaction not found".to_string()));
    }

    Ok(())
}
