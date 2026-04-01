use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{ReadingStatus, ReadingStatusType};

pub async fn get_my_statuses(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<ReadingStatus>, AppError> {
    let statuses = sqlx::query_as::<_, ReadingStatus>(
        "SELECT * FROM up_reading_status WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(statuses)
}

pub async fn get_statuses() -> Vec<ReadingStatusType> {
    vec![
        ReadingStatusType::WantToRead,
        ReadingStatusType::Reading,
        ReadingStatusType::Read,
    ]
}

pub async fn set_reading_status(
    pool: &PgPool,
    user_id: Uuid,
    book_id: Uuid,
    status: ReadingStatusType,
) -> Result<ReadingStatus, AppError> {
    let record = sqlx::query_as::<_, ReadingStatus>(
        r#"
        INSERT INTO up_reading_status (id, user_id, book_id, status)
        VALUES (gen_random_uuid(), $1, $2, $3)
        ON CONFLICT (user_id, book_id) DO UPDATE
            SET status = EXCLUDED.status,
                updated_at = now()
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(book_id)
    .bind(status)
    .fetch_one(pool)
    .await?;

    Ok(record)
}
