//! Database models for users, orders, and stakes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    #[serde(skip_serializing)]
    pub wallet_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub idempotency_key: Option<String>,
    pub stock_symbol: String,
    pub side: String, // "buy" | "sell"
    pub quantity: i64,
    pub price_per_unit: rust_decimal::Decimal,
    pub status: String, // "pending" | "filled" | "cancelled" | "failed"
    pub tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Stake {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: rust_decimal::Decimal,
    pub locked_until: DateTime<Utc>,
    pub reward_claimed: rust_decimal::Decimal,
    pub tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

// Re-export Decimal for convenience - sqlx uses rust_decimal
pub use rust_decimal::Decimal;
