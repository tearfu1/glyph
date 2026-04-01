use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{PublicUser, ReviewWithUser, UpdateUser, User, UserNavigation, UserRole};

pub async fn get_user_profile(pool: &PgPool, user_id: Uuid) -> Result<PublicUser, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM up_user WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

    Ok(user.into())
}

pub async fn get_user_settings(pool: &PgPool, user_id: Uuid) -> Result<User, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM up_user WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

    Ok(user)
}

pub async fn update_user(
    pool: &PgPool,
    user_id: Uuid,
    input: UpdateUser,
) -> Result<PublicUser, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE up_user
        SET
            display_name = COALESCE($1, display_name),
            avatar_url   = COALESCE($2, avatar_url),
            updated_at   = now()
        WHERE id = $3
        RETURNING *
        "#,
    )
    .bind(input.display_name)
    .bind(input.avatar_url)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

    Ok(user.into())
}

pub async fn get_user_navigation(pool: &PgPool, user_id: Uuid) -> Result<UserNavigation, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM up_user WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

    let review_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM up_review WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let question_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM up_question WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(UserNavigation {
        role: user.role,
        review_count,
        question_count,
    })
}

const PAGE_SIZE: i64 = 20;

pub async fn get_user_reviews(
    pool: &PgPool,
    user_id: Uuid,
    page: i64,
) -> Result<(Vec<ReviewWithUser>, i64), AppError> {
    let offset = (page.max(1) - 1) * PAGE_SIZE;

    let rows = sqlx::query_as::<_, ReviewWithUser>(
        r#"
        SELECT
            r.id, r.book_id, r.user_id, r.rating, r.text, r.created_at, r.updated_at,
            u.login      AS user_login,
            u.display_name AS user_display_name,
            u.avatar_url AS user_avatar_url,
            COALESCE(SUM(CASE WHEN rr.is_like = true  THEN 1 ELSE 0 END), 0) AS like_count,
            COALESCE(SUM(CASE WHEN rr.is_like = false THEN 1 ELSE 0 END), 0) AS dislike_count,
            NULL::boolean AS user_reaction
        FROM up_review r
        JOIN up_user u ON u.id = r.user_id
        LEFT JOIN up_review_reaction rr ON rr.review_id = r.id
        WHERE r.user_id = $1
        GROUP BY r.id, u.login, u.display_name, u.avatar_url
        ORDER BY r.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(PAGE_SIZE)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM up_review WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

pub async fn get_all_groups(_pool: &PgPool) -> Vec<UserRole> {
    vec![
        UserRole::User,
        UserRole::Premium,
        UserRole::Author,
        UserRole::Admin,
    ]
}

pub async fn update_user_groups(
    pool: &PgPool,
    user_id: Uuid,
    role: UserRole,
) -> Result<PublicUser, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE up_user
        SET role = $1, updated_at = now()
        WHERE id = $2
        RETURNING *
        "#,
    )
    .bind(role)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

    Ok(user.into())
}
