use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use std::sync::Arc;

use crate::auth::jwt::{validate_token, Claims};
use crate::config::AppConfig;
use crate::errors::AppError;

#[derive(Debug, Clone)]
pub struct AuthUser(pub Claims);

#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<Claims>);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let config = parts
            .extensions
            .get::<Arc<AppConfig>>()
            .ok_or(AppError::Unauthorized)?;

        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        let claims = validate_token(token, &config.jwt_secret)?;
        Ok(AuthUser(claims))
    }
}

impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let config = parts
            .extensions
            .get::<Arc<AppConfig>>();

        let claims = config.and_then(|config| {
            let auth_header = parts
                .headers
                .get("Authorization")
                .and_then(|v| v.to_str().ok())?;
            let token = auth_header.strip_prefix("Bearer ")?;
            validate_token(token, &config.jwt_secret).ok()
        });

        Ok(OptionalAuthUser(claims))
    }
}
