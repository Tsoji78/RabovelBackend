//! User routes - profile (JWT protected).

use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::auth_user;
use crate::config::Config;
use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub user_id: String,
    pub email: String,
    pub wallet_address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub wallet_address: Option<String>,
}

pub async fn profile(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let auth = auth_user(&req, config.get_ref())?;

    let row = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT id::text, email, wallet_address FROM users WHERE id = $1",
    )
    .bind(auth.user_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ProfileResponse {
        user_id: row.0,
        email: row.1,
        wallet_address: row.2,
    }))
}

pub async fn update_profile(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    req: HttpRequest,
    body: web::Json<UpdateProfileRequest>,
) -> Result<HttpResponse, AppError> {
    let auth = auth_user(&req, config.get_ref())?;

    if let Some(ref addr) = body.wallet_address {
        if addr.len() != 42 || !addr.starts_with("0x") {
            return Err(AppError::Validation(
                "wallet_address must be 42 characters (0x + 40 hex)".to_string(),
            ));
        }
        sqlx::query("UPDATE users SET wallet_address = $1, updated_at = NOW() WHERE id = $2")
            .bind(addr)
            .bind(auth.user_id)
            .execute(pool.get_ref())
            .await?;
    }

    let row = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT id::text, email, wallet_address FROM users WHERE id = $1",
    )
    .bind(auth.user_id)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ProfileResponse {
        user_id: row.0,
        email: row.1,
        wallet_address: row.2,
    }))
}
