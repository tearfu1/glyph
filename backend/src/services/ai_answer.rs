use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{AiAnswer, AiAnswerSource};

#[derive(Debug, Serialize)]
struct MlRequest {
    question: String,
    author: String,
    book_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct MlSource {
    text: String,
    score: f64,
    book: String,
}

#[derive(Debug, Deserialize)]
struct MlResponse {
    answer: String,
    sources: Vec<MlSource>,
}

pub async fn generate_ai_answer(
    pool: &PgPool,
    ml_service_url: &str,
    question_id: Uuid,
) -> Result<AiAnswer, AppError> {
    // Check that there is no existing AI answer
    let already_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM up_ai_answer WHERE question_id = $1)",
    )
    .bind(question_id)
    .fetch_one(pool)
    .await?;

    if already_exists {
        return Err(AppError::Conflict(
            "AI-ответ для этого вопроса уже существует".to_string(),
        ));
    }

    #[derive(FromRow)]
    struct QuestionContext {
        question_text: String,
        book_id: Uuid,
        author_name: String,
    }

    // Fetch question text and book/author info
    let row = sqlx::query_as::<_, QuestionContext>(
        r#"
        SELECT q.text AS question_text,
               q.book_id,
               u.display_name AS author_name
        FROM up_question q
        JOIN up_book b ON b.id = q.book_id
        JOIN up_user u ON u.id = b.author_id
        WHERE q.id = $1
        "#,
    )
    .bind(question_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Вопрос {} не найден", question_id)))?;

    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("Failed to build HTTP client: {}", e)))?;

    let ml_url = format!("{}/api/generate-answer", ml_service_url);

    let ml_req = MlRequest {
        question: row.question_text,
        author: row.author_name,
        book_id: Some(row.book_id),
    };

    let ml_resp = client
        .post(&ml_url)
        .json(&ml_req)
        .send()
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("ML service request failed: {}", e)))?;

    if !ml_resp.status().is_success() {
        let status = ml_resp.status();
        let body = ml_resp.text().await.unwrap_or_default();
        return Err(AppError::InternalError(anyhow::anyhow!(
            "ML service returned {}: {}",
            status,
            body
        )));
    }

    let ml_data: MlResponse = ml_resp
        .json()
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("Failed to parse ML response: {}", e)))?;

    let sources: Vec<AiAnswerSource> = ml_data
        .sources
        .into_iter()
        .map(|s| AiAnswerSource {
            text: s.text,
            score: s.score,
            book: s.book,
        })
        .collect();

    let sources_json = serde_json::to_value(&sources)
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("Failed to serialize sources: {}", e)))?;

    let ai_answer = sqlx::query_as::<_, AiAnswer>(
        r#"
        INSERT INTO up_ai_answer (id, question_id, answer_text, sources)
        VALUES (gen_random_uuid(), $1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(question_id)
    .bind(&ml_data.answer)
    .bind(sources_json)
    .fetch_one(pool)
    .await?;

    Ok(ai_answer)
}

pub async fn get_ai_answer(
    pool: &PgPool,
    question_id: Uuid,
) -> Result<AiAnswer, AppError> {
    sqlx::query_as::<_, AiAnswer>(
        "SELECT * FROM up_ai_answer WHERE question_id = $1",
    )
    .bind(question_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("AI-ответ не найден".to_string()))
}
