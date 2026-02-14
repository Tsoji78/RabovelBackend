# Rabovel Smart Contracts

Solidity contracts for the blockchain stock broker and staking platform.

## Setup (Foundry)

```bash
# Install Foundry (if not installed)
curl -L https://foundry.paradigm.xyz | bash
foundryup

# In contracts directory
cd contracts

# Install OpenZeppelin
forge install OpenZeppelin/openzeppelin-contracts --no-commit
```

## Build

```bash
forge build
```

## Test

```bash
forge test
```

## Deploy (Sepolia testnet example)

```bash
# Set env vars
export RPC_URL="https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY"
export PRIVATE_KEY="0x..."

# Deploy
forge script script/Deploy.s.sol --rpc-url $RPC_URL --broadcast --verify
```

## Contracts

| Contract        | Description                                              |
|----------------|----------------------------------------------------------|
| PlatformToken  | ERC-20 platform token (RABO)                             |
| SecurityToken  | ERC-1400-style security token for tokenized stocks       |
| Staking        | Lock/unlock staking with reward logic                    |
| Trading        | Order placement and matching placeholder                |
