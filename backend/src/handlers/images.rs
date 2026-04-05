use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::errors::AppError;
use crate::AppState;

#[derive(Serialize)]
pub struct UploadResponse {
    pub url: String,
}

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10 MB
const ALLOWED_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp", "image/gif"];

pub async fn upload_image(
    State(state): State<AppState>,
    _auth: AuthUser,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), AppError> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Invalid multipart: {e}")))?
        .ok_or_else(|| AppError::BadRequest("No file provided".to_string()))?;

    let content_type = field
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();

    if !ALLOWED_TYPES.contains(&content_type.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Unsupported file type: {content_type}. Allowed: JPEG, PNG, WebP, GIF"
        )));
    }

    let extension = match content_type.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "image/gif" => "gif",
        _ => "bin",
    };

    let data = field
        .bytes()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read file: {e}")))?;

    if data.len() > MAX_FILE_SIZE {
        return Err(AppError::BadRequest(format!(
            "File too large: {} bytes (max {})",
            data.len(),
            MAX_FILE_SIZE
        )));
    }

    let filename = format!("{}.{}", Uuid::new_v4(), extension);
    let upload_dir = &state.config.upload_dir;

    tokio::fs::create_dir_all(upload_dir)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("Failed to create upload dir: {e}")))?;

    let file_path = format!("{}/{}", upload_dir, filename);
    tokio::fs::write(&file_path, &data)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("Failed to save file: {e}")))?;

    let url = format!("/uploads/{}", filename);

    Ok((StatusCode::CREATED, Json(UploadResponse { url })))
}
