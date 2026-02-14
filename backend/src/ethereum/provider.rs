//! Ethereum provider setup - HTTP/WS connection to RPC.

use ethers::providers::{Http, Provider};
use std::sync::Arc;
use crate::config::Config;
use crate::error::AppError;

/// Shared Ethereum provider (HTTP). Clone via Arc for use across handlers.
pub type EthProvider = Arc<Provider<Http>>;

/// Create provider from config RPC_URL.
pub async fn create_provider(config: &Config) -> Result<EthProvider, AppError> {
    let provider = Provider::<Http>::try_from(&config.rpc_url)
        .map_err(|e| AppError::Ethereum(format!("Failed to connect to RPC: {}", e)))?;
    Ok(Arc::new(provider))
}
