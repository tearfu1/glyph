use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{Book, BookWithAuthor, CreateBook, ReadingStatusType, UpdateBook};

const PAGE_SIZE: i64 = 10;

pub async fn get_books(
    pool: &PgPool,
    page: i64,
    search: Option<String>,
    tag_ids: Option<Vec<Uuid>>,
) -> Result<(Vec<BookWithAuthor>, i64), AppError> {
    let offset = (page.max(1) - 1) * PAGE_SIZE;

    // Build query dynamically based on filters
    let rows = match (&search, &tag_ids) {
        (Some(q), Some(tags)) if !tags.is_empty() => {
            let pattern = format!("%{}%", q);
            sqlx::query_as::<_, BookWithAuthor>(
                r#"
                SELECT
                    b.id, b.title, b.description, b.cover_url, b.isbn,
                    b.published_year, b.author_id, b.created_at, b.updated_at,
                    u.login AS author_login,
                    u.email AS author_email,
                    u.display_name AS author_display_name,
                    u.avatar_url AS author_avatar_url
                FROM up_book b
                JOIN up_user u ON u.id = b.author_id
                WHERE (b.title ILIKE $1 OR u.display_name ILIKE $1)
                  AND (
                      SELECT COUNT(*) FROM up_book_tag bt
                      WHERE bt.book_id = b.id AND bt.tag_id = ANY($2)
                  ) = array_length($2, 1)
                ORDER BY b.created_at DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(&pattern)
            .bind(tags)
            .bind(PAGE_SIZE)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
        (Some(q), _) => {
            let pattern = format!("%{}%", q);
            sqlx::query_as::<_, BookWithAuthor>(
                r#"
                SELECT
                    b.id, b.title, b.description, b.cover_url, b.isbn,
                    b.published_year, b.author_id, b.created_at, b.updated_at,
                    u.login AS author_login,
                    u.email AS author_email,
                    u.display_name AS author_display_name,
                    u.avatar_url AS author_avatar_url
                FROM up_book b
                JOIN up_user u ON u.id = b.author_id
                WHERE b.title ILIKE $1 OR u.display_name ILIKE $1
                ORDER BY b.created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(&pattern)
            .bind(PAGE_SIZE)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
        (None, Some(tags)) if !tags.is_empty() => {
            sqlx::query_as::<_, BookWithAuthor>(
                r#"
                SELECT
                    b.id, b.title, b.description, b.cover_url, b.isbn,
                    b.published_year, b.author_id, b.created_at, b.updated_at,
                    u.login AS author_login,
                    u.email AS author_email,
                    u.display_name AS author_display_name,
                    u.avatar_url AS author_avatar_url
                FROM up_book b
                JOIN up_user u ON u.id = b.author_id
                WHERE (
                    SELECT COUNT(*) FROM up_book_tag bt
                    WHERE bt.book_id = b.id AND bt.tag_id = ANY($1)
                ) = array_length($1, 1)
                ORDER BY b.created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(tags)
            .bind(PAGE_SIZE)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
        _ => {
            sqlx::query_as::<_, BookWithAuthor>(
                r#"
                SELECT
                    b.id, b.title, b.description, b.cover_url, b.isbn,
                    b.published_year, b.author_id, b.created_at, b.updated_at,
                    u.login AS author_login,
                    u.email AS author_email,
                    u.display_name AS author_display_name,
                    u.avatar_url AS author_avatar_url
                FROM up_book b
                JOIN up_user u ON u.id = b.author_id
                ORDER BY b.created_at DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(PAGE_SIZE)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
    };

    let total = match (&search, &tag_ids) {
        (Some(q), Some(tags)) if !tags.is_empty() => {
            let pattern = format!("%{}%", q);
            sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(*) FROM up_book b
                JOIN up_user u ON u.id = b.author_id
                WHERE (b.title ILIKE $1 OR u.display_name ILIKE $1)
                  AND (
                      SELECT COUNT(*) FROM up_book_tag bt
                      WHERE bt.book_id = b.id AND bt.tag_id = ANY($2)
                  ) = array_length($2, 1)
                "#,
            )
            .bind(&pattern)
            .bind(tags)
            .fetch_one(pool)
            .await?
        }
        (Some(q), _) => {
            let pattern = format!("%{}%", q);
            sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(*) FROM up_book b
                JOIN up_user u ON u.id = b.author_id
                WHERE b.title ILIKE $1 OR u.display_name ILIKE $1
                "#,
            )
            .bind(&pattern)
            .fetch_one(pool)
            .await?
        }
        (None, Some(tags)) if !tags.is_empty() => {
            sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(*) FROM up_book b
                WHERE (
                    SELECT COUNT(*) FROM up_book_tag bt
                    WHERE bt.book_id = b.id AND bt.tag_id = ANY($1)
                ) = array_length($1, 1)
                "#,
            )
            .bind(tags)
            .fetch_one(pool)
            .await?
        }
        _ => {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM up_book")
                .fetch_one(pool)
                .await?
        }
    };

    Ok((rows, total))
}

pub async fn get_book_by_id(pool: &PgPool, id: Uuid) -> Result<BookWithAuthor, AppError> {
    let book = sqlx::query_as::<_, BookWithAuthor>(
        r#"
        SELECT
            b.id, b.title, b.description, b.cover_url, b.isbn,
            b.published_year, b.author_id, b.created_at, b.updated_at,
            u.login AS author_login,
            u.email AS author_email,
            u.display_name AS author_display_name,
            u.avatar_url AS author_avatar_url
        FROM up_book b
        JOIN up_user u ON u.id = b.author_id
        WHERE b.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Book {} not found", id)))?;

    Ok(book)
}

pub async fn create_book(
    pool: &PgPool,
    author_id: Uuid,
    input: CreateBook,
) -> Result<Book, AppError> {
    let book = sqlx::query_as::<_, Book>(
        r#"
        INSERT INTO up_book (id, title, description, cover_url, isbn, published_year, author_id)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(&input.title)
    .bind(&input.description)
    .bind(&input.cover_url)
    .bind(&input.isbn)
    .bind(input.published_year)
    .bind(author_id)
    .fetch_one(pool)
    .await?;

    Ok(book)
}

pub async fn update_book(
    pool: &PgPool,
    id: Uuid,
    input: UpdateBook,
) -> Result<Book, AppError> {
    let book = sqlx::query_as::<_, Book>(
        r#"
        UPDATE up_book SET
            title = COALESCE($2, title),
            description = COALESCE($3, description),
            cover_url = COALESCE($4, cover_url),
            isbn = COALESCE($5, isbn),
            published_year = COALESCE($6, published_year),
            updated_at = now()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(&input.title)
    .bind(&input.description)
    .bind(&input.cover_url)
    .bind(&input.isbn)
    .bind(input.published_year)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Book {} not found", id)))?;

    Ok(book)
}

pub async fn get_books_by_author(
    pool: &PgPool,
    author_id: Uuid,
    page: i64,
) -> Result<(Vec<Book>, i64), AppError> {
    let offset = (page.max(1) - 1) * PAGE_SIZE;

    let rows = sqlx::query_as::<_, Book>(
        r#"
        SELECT * FROM up_book
        WHERE author_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(author_id)
    .bind(PAGE_SIZE)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM up_book WHERE author_id = $1",
    )
    .bind(author_id)
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

pub async fn get_books_for_shelf(
    pool: &PgPool,
    user_id: Uuid,
    status: Option<ReadingStatusType>,
    page: i64,
) -> Result<(Vec<BookWithAuthor>, i64), AppError> {
    let offset = (page.max(1) - 1) * PAGE_SIZE;

    let rows = match &status {
        Some(s) => {
            sqlx::query_as::<_, BookWithAuthor>(
                r#"
                SELECT
                    b.id, b.title, b.description, b.cover_url, b.isbn,
                    b.published_year, b.author_id, b.created_at, b.updated_at,
                    u.login AS author_login,
                    u.email AS author_email,
                    u.display_name AS author_display_name,
                    u.avatar_url AS author_avatar_url
                FROM up_book b
                JOIN up_user u ON u.id = b.author_id
                JOIN up_reading_status rs ON rs.book_id = b.id
                WHERE rs.user_id = $1 AND rs.status = $2
                ORDER BY rs.updated_at DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(user_id)
            .bind(s)
            .bind(PAGE_SIZE)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, BookWithAuthor>(
                r#"
                SELECT
                    b.id, b.title, b.description, b.cover_url, b.isbn,
                    b.published_year, b.author_id, b.created_at, b.updated_at,
                    u.login AS author_login,
                    u.email AS author_email,
                    u.display_name AS author_display_name,
                    u.avatar_url AS author_avatar_url
                FROM up_book b
                JOIN up_user u ON u.id = b.author_id
                JOIN up_reading_status rs ON rs.book_id = b.id
                WHERE rs.user_id = $1
                ORDER BY rs.updated_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(user_id)
            .bind(PAGE_SIZE)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
    };

    let total = match &status {
        Some(s) => {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM up_reading_status WHERE user_id = $1 AND status = $2",
            )
            .bind(user_id)
            .bind(s)
            .fetch_one(pool)
            .await?
        }
        None => {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM up_reading_status WHERE user_id = $1",
            )
            .bind(user_id)
            .fetch_one(pool)
            .await?
        }
    };

    Ok((rows, total))
}
