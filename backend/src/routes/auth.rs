//! Auth routes - register, login.

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::auth::{create_token, hash_password, verify_password};
use crate::config::Config;
use crate::db::User;
use crate::error::AppError;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    pub wallet_address: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

pub async fn register(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let existing = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&body.email)
        .fetch_one(pool.get_ref())
        .await?;

    if existing > 0 {
        return Err(AppError::Validation("Email already registered".to_string()));
    }

    let password_hash = hash_password(&body.password)?;
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, wallet_address, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $5)
        "#,
    )
    .bind(id)
    .bind(&body.email)
    .bind(&password_hash)
    .bind(&body.wallet_address)
    .bind(now)
    .execute(pool.get_ref())
    .await?;

    let token = create_token(id, &body.email, &config.jwt_secret, config.jwt_expiry_hours)?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user_id: id.to_string(),
        email: body.email.clone(),
    }))
}

pub async fn login(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&body.email)
        .fetch_optional(pool.get_ref())
        .await?
        .ok_or_else(|| AppError::Auth("Invalid email or password".to_string()))?;

    let valid = verify_password(&body.password, &user.password_hash)
        .map_err(|_| AppError::Auth("Invalid email or password".to_string()))?;

    if !valid {
        return Err(AppError::Auth("Invalid email or password".to_string()));
    }

    let token = create_token(user.id, &user.email, &config.jwt_secret, config.jwt_expiry_hours)?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user_id: user.id.to_string(),
        email: user.email,
    }))
}
