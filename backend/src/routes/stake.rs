//! Staking routes - lock, claim rewards.

use actix_web::{web, HttpRequest, HttpResponse};
use ethers::types::Address;
use rust_decimal::Decimal;
use rust_decimal::prelude::Zero;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;
use validator::Validate;

use crate::auth::auth_user;
use crate::config::Config;
use crate::ethereum::{get_pending_rewards, get_staked_balance, EthProvider};
use crate::error::AppError;

#[derive(Debug, Deserialize, Validate)]
pub struct LockRequest {
    #[validate(range(min = 0))]
    pub amount: Decimal,
    pub lock_days: u32,
}

#[derive(Debug, Serialize)]
pub struct StakeResponse {
    pub stake_id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ClaimResponse {
    pub amount: String,
}

#[derive(Debug, Serialize)]
pub struct StakingPositionResponse {
    pub staked_balance: String,
    pub pending_rewards: String,
}

pub async fn lock(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    provider: web::Data<EthProvider>,
    req: HttpRequest,
    body: web::Json<LockRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let auth = auth_user(&req, config.get_ref())?;

    let wallet = sqlx::query_scalar::<_, Option<String>>("SELECT wallet_address FROM users WHERE id = $1")
        .bind(auth.user_id)
        .fetch_one(pool.get_ref())
        .await?
        .flatten()
        .ok_or_else(|| AppError::Validation("Wallet address required for staking".to_string()))?;

    let staking_addr = config
        .staking_contract_address
        .as_ref()
        .ok_or_else(|| AppError::Ethereum("Staking contract not configured".to_string()))?;
    let addr = Address::from_str(staking_addr)
        .map_err(|_| AppError::Validation("Invalid staking contract address".to_string()))?;

    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let locked_until = now + chrono::Duration::days(i64::from(body.lock_days));

    sqlx::query(
        r#"
        INSERT INTO stakes (id, user_id, amount, locked_until, reward_claimed, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(id)
    .bind(auth.user_id)
    .bind(body.amount)
    .bind(locked_until)
    .bind(Decimal::zero())
    .bind(now)
    .execute(pool.get_ref())
    .await?;

    // TODO: Call staking contract stake() via signer; store tx_hash on confirm
    // For now we persist the intent. Event listener or cron will sync on-chain state.
    let _ = (provider, wallet, addr); // suppress unused when contract call not wired

    Ok(HttpResponse::Accepted().json(StakeResponse {
        stake_id: id.to_string(),
        status: "pending".to_string(),
    }))
}

pub async fn claim_rewards(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    provider: web::Data<EthProvider>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let auth = auth_user(&req, config.get_ref())?;

    let wallet = sqlx::query_scalar::<_, Option<String>>("SELECT wallet_address FROM users WHERE id = $1")
        .bind(auth.user_id)
        .fetch_one(pool.get_ref())
        .await?
        .flatten()
        .ok_or_else(|| AppError::Validation("Wallet address required".to_string()))?;

    let staking_addr = config
        .staking_contract_address
        .as_ref()
        .ok_or_else(|| AppError::Ethereum("Staking contract not configured".to_string()))?;
    let addr = Address::from_str(staking_addr)
        .map_err(|_| AppError::Validation("Invalid staking contract address".to_string()))?;
    let account = Address::from_str(&wallet)
        .map_err(|_| AppError::Validation("Invalid wallet address".to_string()))?;

    let pending = get_pending_rewards(provider.get_ref(), addr, account).await?;

    // TODO: Call contract claimRewards() via signer; persist tx_hash
    let _ = pool;

    Ok(HttpResponse::Ok().json(ClaimResponse {
        amount: pending.to_string(),
    }))
}

pub async fn position(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    provider: web::Data<EthProvider>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let auth = auth_user(&req, config.get_ref())?;

    let wallet = sqlx::query_scalar::<_, Option<String>>("SELECT wallet_address FROM users WHERE id = $1")
        .bind(auth.user_id)
        .fetch_one(pool.get_ref())
        .await?
        .flatten()
        .ok_or_else(|| AppError::Validation("Wallet address required".to_string()))?;

    let staking_addr = config
        .staking_contract_address
        .as_ref()
        .ok_or_else(|| AppError::Ethereum("Staking contract not configured".to_string()))?;
    let addr = Address::from_str(staking_addr)
        .map_err(|_| AppError::Validation("Invalid staking contract address".to_string()))?;
    let account = Address::from_str(&wallet)
        .map_err(|_| AppError::Validation("Invalid wallet address".to_string()))?;

    let staked = get_staked_balance(provider.get_ref(), addr, account).await?;
    let rewards = get_pending_rewards(provider.get_ref(), addr, account).await?;

    Ok(HttpResponse::Ok().json(StakingPositionResponse {
        staked_balance: staked.to_string(),
        pending_rewards: rewards.to_string(),
    }))
}
