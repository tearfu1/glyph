use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use time::Duration;
use uuid::Uuid;
use validator::Validate;

use crate::auth::jwt::{create_access_token, create_refresh_token, validate_token};
use crate::auth::password::{hash_password, verify_password};
use crate::errors::AppError;
use crate::models::{CreateUser, PublicUser, User};
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub user: PublicUser,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(input): Json<CreateUser>,
) -> Result<(CookieJar, (StatusCode, Json<AuthResponse>)), AppError> {
    input
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM up_user WHERE email = $1 OR login = $2)",
    )
    .bind(&input.email)
    .bind(&input.login)
    .fetch_one(&state.pool)
    .await?;

    if exists {
        return Err(AppError::Conflict(
            "User with this email or login already exists".to_string(),
        ));
    }

    let password_hash = hash_password(&input.password)?;
    let user_id = Uuid::new_v4();

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO up_user (id, login, email, password_hash, display_name, role)
        VALUES ($1, $2, $3, $4, $5, 'user')
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(&input.login)
    .bind(&input.email)
    .bind(&password_hash)
    .bind(&input.display_name)
    .fetch_one(&state.pool)
    .await?;

    let access_token = create_access_token(
        user.id,
        user.role.clone(),
        &state.config.jwt_secret,
        state.config.jwt_access_expiration_secs,
    )?;
    let refresh_token = create_refresh_token(
        user.id,
        user.role.clone(),
        &state.config.jwt_secret,
        state.config.jwt_refresh_expiration_secs,
    )?;

    let cookie = build_refresh_cookie(refresh_token, state.config.jwt_refresh_expiration_secs);

    Ok((
        jar.add(cookie),
        (StatusCode::CREATED, Json(AuthResponse {
            access_token,
            user: user.into(),
        })),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(input): Json<LoginRequest>,
) -> Result<(CookieJar, Json<AuthResponse>), AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM up_user WHERE email = $1")
        .bind(&input.email)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !verify_password(&input.password, &user.password_hash)? {
        return Err(AppError::Unauthorized);
    }

    let access_token = create_access_token(
        user.id,
        user.role.clone(),
        &state.config.jwt_secret,
        state.config.jwt_access_expiration_secs,
    )?;
    let refresh_token = create_refresh_token(
        user.id,
        user.role.clone(),
        &state.config.jwt_secret,
        state.config.jwt_refresh_expiration_secs,
    )?;

    let cookie = build_refresh_cookie(refresh_token, state.config.jwt_refresh_expiration_secs);

    Ok((
        jar.add(cookie),
        Json(AuthResponse {
            access_token,
            user: user.into(),
        }),
    ))
}

pub async fn refresh(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<AuthResponse>), AppError> {
    let refresh_token = jar
        .get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;

    let claims = validate_token(&refresh_token, &state.config.jwt_secret)?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM up_user WHERE id = $1")
        .bind(claims.sub)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let access_token = create_access_token(
        user.id,
        user.role.clone(),
        &state.config.jwt_secret,
        state.config.jwt_access_expiration_secs,
    )?;
    let new_refresh_token = create_refresh_token(
        user.id,
        user.role.clone(),
        &state.config.jwt_secret,
        state.config.jwt_refresh_expiration_secs,
    )?;

    let cookie = build_refresh_cookie(new_refresh_token, state.config.jwt_refresh_expiration_secs);

    Ok((
        jar.add(cookie),
        Json(AuthResponse {
            access_token,
            user: user.into(),
        }),
    ))
}

fn build_refresh_cookie(token: String, max_age_secs: i64) -> Cookie<'static> {
    Cookie::build(("refresh_token", token))
        .path("/api/auth")
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(Duration::seconds(max_age_secs))
        .build()
}
