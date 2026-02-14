//! Event listener for on-chain confirmations.
//! Listen to Staked, OrderPlaced, OrderFilled events and update DB (tx_hash, status).
//! Production: spawn as background task in main().

use tracing::info;

use crate::config::Config;
use crate::error::AppError;

/// Placeholder: spawn event listener for staking/trading events.
/// Example flow: subscribe to Staked events -> update stakes.tx_hash when confirmed.
pub async fn spawn_event_listener(_config: &Config) -> Result<(), AppError> {
    // TODO: Use WS provider for subscribe
    // let ws = Provider::<Ws>::connect(&config.rpc_url.replace("https", "wss")).await?;
    // let contract = IStaking::new(staking_addr, &ws);
    // contract.events().staked().subscribe().await?;
    // On each event: update stakes table tx_hash
    info!("Event listener not started (configure WS RPC for subscriptions)");
    Ok(())
}
