//! JWT extractor for protected routes.

use actix_web::{http::header, HttpRequest};
use uuid::Uuid;
use crate::auth::jwt::validate_token;
use crate::config::Config;
use crate::error::AppError;

/// Authenticated user context for protected handlers.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
}

/// Extract AuthUser from Bearer token in Authorization header.
/// Call from protected handlers: `let auth = auth_user(&req, &config)?;`
pub fn auth_user(req: &HttpRequest, config: &Config) -> Result<AuthUser, AppError> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Auth("Missing Authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Auth("Invalid Authorization format".to_string()))?;

    let claims = validate_token(token, &config.jwt_secret)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Auth("Invalid user id in token".to_string()))?;

    Ok(AuthUser {
        user_id,
        email: claims.email,
    })
}
