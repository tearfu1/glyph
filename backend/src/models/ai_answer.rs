use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAnswerSource {
    pub text: String,
    pub score: f64,
    pub book: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiAnswer {
    pub id: Uuid,
    pub question_id: Uuid,
    pub answer_text: String,
    pub sources: sqlx::types::Json<Vec<AiAnswerSource>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AiAnswerResponse {
    pub id: Uuid,
    pub question_id: Uuid,
    pub answer_text: String,
    pub sources: Vec<AiAnswerSource>,
    pub created_at: DateTime<Utc>,
}

impl From<AiAnswer> for AiAnswerResponse {
    fn from(a: AiAnswer) -> Self {
        Self {
            id: a.id,
            question_id: a.question_id,
            answer_text: a.answer_text,
            sources: a.sources.0,
            created_at: a.created_at,
        }
    }
}
