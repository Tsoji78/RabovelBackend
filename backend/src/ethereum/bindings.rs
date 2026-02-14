//! Contract bindings - Staking, ERC20.
//! Use ethers abigen! macro. Production: use ABI JSON from compiled artifacts.

use ethers::contract::abigen;
use ethers::types::{Address, U256};
use crate::ethereum::EthProvider;
use crate::error::AppError;

abigen!(
    IERC20,
    r"[
        function balanceOf(address) view returns (uint256)
        function allowance(address,address) view returns (uint256)
    ]"
);

abigen!(
    IStaking,
    r"[
        function getStakedBalance(address) view returns (uint256)
        function getPendingRewards(address) view returns (uint256)
        event Staked(address indexed user, uint256 amount)
        event Unstaked(address indexed user, uint256 amount)
        event RewardsClaimed(address indexed user, uint256 amount)
    ]"
);

/// Read staked balance for an address.
pub async fn get_staked_balance(
    provider: &EthProvider,
    staking_address: Address,
    account: Address,
) -> Result<U256, AppError> {
    let contract = IStaking::new(staking_address, provider.clone());
    contract
        .get_staked_balance(account)
        .call()
        .await
        .map_err(|e| AppError::Ethereum(format!("Staking call failed: {}", e)))
}

/// Read pending rewards for an address.
pub async fn get_pending_rewards(
    provider: &EthProvider,
    staking_address: Address,
    account: Address,
) -> Result<U256, AppError> {
    let contract = IStaking::new(staking_address, provider.clone());
    contract
        .get_pending_rewards(account)
        .call()
        .await
        .map_err(|e| AppError::Ethereum(format!("Rewards call failed: {}", e)))
}

/// Read ERC20 balance.
pub async fn get_erc20_balance(
    provider: &EthProvider,
    token_address: Address,
    account: Address,
) -> Result<U256, AppError> {
    let contract = IERC20::new(token_address, provider.clone());
    contract
        .balance_of(account)
        .call()
        .await
        .map_err(|e| AppError::Ethereum(format!("Balance call failed: {}", e)))
}
