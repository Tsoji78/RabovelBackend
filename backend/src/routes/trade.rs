//! Trade routes - place order (with idempotency).

use actix_web::{web, HttpRequest, HttpResponse};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::auth::auth_user;
use crate::config::Config;
use crate::error::AppError;

#[derive(Debug, Deserialize, Validate)]
pub struct PlaceOrderRequest {
    #[validate(length(min = 1, max = 64))]
    pub idempotency_key: Option<String>,
    #[validate(length(min = 1, max = 20))]
    pub stock_symbol: String,
    #[validate(length(min = 1))]
    pub side: String,
    #[validate(range(min = 1))]
    pub quantity: i64,
    pub price_per_unit: Decimal,
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub status: String,
}

pub async fn place_order(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    req: HttpRequest,
    body: web::Json<PlaceOrderRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let side = body.side.to_lowercase();
    if side != "buy" && side != "sell" {
        return Err(AppError::Validation("Side must be 'buy' or 'sell'".to_string()));
    }

    let auth = auth_user(&req, config.get_ref())?;

    // Idempotency: if key provided and order exists, return existing order
    if let Some(ref key) = body.idempotency_key {
        if let Some(existing) = sqlx::query_as::<_, (Uuid, String)>(
            "SELECT id, status FROM orders WHERE idempotency_key = $1 AND user_id = $2",
        )
        .bind(key)
        .bind(auth.user_id)
        .fetch_optional(pool.get_ref())
        .await?
        {
            return Ok(HttpResponse::Ok().json(OrderResponse {
                order_id: existing.0.to_string(),
                status: existing.1,
            }));
        }
    }

    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query(
        r#"
        INSERT INTO orders (id, user_id, idempotency_key, stock_symbol, side, quantity, price_per_unit, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', $8, $8)
        "#,
    )
    .bind(id)
    .bind(auth.user_id)
    .bind(&body.idempotency_key)
    .bind(&body.stock_symbol)
    .bind(&side)
    .bind(body.quantity)
    .bind(body.price_per_unit)
    .bind(now)
    .execute(pool.get_ref())
    .await?;

    // TODO: Submit to trading contract; listen for OrderPlaced event; update tx_hash
    // For now we persist off-chain and return. Event listener will update status.

    Ok(HttpResponse::Accepted().json(OrderResponse {
        order_id: id.to_string(),
        status: "pending".to_string(),
    }))
}
