# Rabovel Backend

Blockchain-based stock broker and staking platform backend. Built with **Rust**, **PostgreSQL**, and **Ethereum** integration.

## Tech Stack

- **Web**: actix-web
- **Database**: PostgreSQL + sqlx
- **Blockchain**: ethers-rs (Ethereum RPC, contract calls)
- **Auth**: JWT + Argon2 password hashing
- **Security**: Rate limiting, CORS, input validation
- **Logging**: tracing + tracing-actix-web

## Project Structure

```
RabovelBackend/
├── backend/                 # Rust API server
│   ├── src/
│   │   ├── main.rs          # Entry point, server setup
│   │   ├── config.rs        # Env-based config
│   │   ├── error.rs         # AppError types
│   │   ├── auth/            # JWT, password hashing
│   │   ├── db/              # Pool, models
│   │   ├── ethereum/        # Provider, contract bindings
│   │   └── routes/         # auth, user, trade, stake, stocks
│   ├── migrations/         # sqlx migrations
│   └── Cargo.toml
├── contracts/               # Solidity (Foundry)
│   ├── src/
│   │   ├── PlatformToken.sol   # ERC-20
│   │   ├── SecurityToken.sol   # ERC-1400-style
│   │   ├── Staking.sol         # Lock/unlock + rewards
│   │   └── Trading.sol         # Order matching
│   └── foundry.toml
└── README.md
```

## Setup

### Prerequisites

- Rust (stable)
- PostgreSQL
- Node.js (optional, for Foundry if using npm)

### 1. Database

```bash
createdb rabovel
```

### 2. Install sqlx-cli

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

### 3. Environment

Create `.env` in project root:

```env
DATABASE_URL=postgres://user:password@localhost/rabovel
JWT_SECRET=your-32-char-secret-key-here
RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY
CHAIN_ID=11155111

# Optional - set after deploying contracts
PLATFORM_TOKEN_ADDRESS=0x...
STAKING_CONTRACT_ADDRESS=0x...
TRADING_CONTRACT_ADDRESS=0x...

# Optional overrides
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
CORS_ORIGINS=*
JWT_EXPIRY_HOURS=24
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW_SECS=60
```

### 4. Run Migrations

```bash
cd backend
sqlx migrate run
```

### 5. Build & Run

```bash
cd backend
cargo run
```

Server starts at `http://0.0.0.0:8080` (or `SERVER_PORT`).

## API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | /health | - | Health check |
| POST | /auth/register | - | Register (email, password) |
| POST | /auth/login | - | Login, returns JWT |
| GET | /user/profile | Bearer | Get user profile |
| PATCH | /user/profile | Bearer | Update profile (e.g. wallet_address) |
| POST | /trade/place-order | Bearer | Place order (idempotency key supported) |
| POST | /stake/lock | Bearer | Lock tokens (amount, lock_days) |
| POST | /stake/claim-rewards | Bearer | Claim staking rewards |
| GET | /stake/position | Bearer | Get staked balance + pending rewards |
| GET | /stocks/list | - | List NGX tokenized equities & ETFs (query: type, search, limit, offset) |
| GET | /stocks/view/{id_or_symbol} | - | View single asset details |
| POST | /stocks/choose | - | Choose asset by id or ngx_symbol (returns full detail) |

## Contracts (Foundry)

```bash
cd contracts
forge install OpenZeppelin/openzeppelin-contracts --no-commit
forge build
```

Deploy to Sepolia:

```bash
export RPC_URL="..."
export PRIVATE_KEY="0x..."
forge script script/Deploy.s.sol --rpc-url $RPC_URL --broadcast
```

See [contracts/README.md](contracts/README.md) for details.

## Security Notes

- **Idempotency**: Use `idempotency_key` on place-order to avoid duplicate orders
- **Wallet**: Users must set `wallet_address` for staking (via profile update)
- **Reentrancy**: Smart contracts use ReentrancyGuard; off-chain logic avoids reentrant patterns
- **Rate limiting**: Per-IP; tune via `RATE_LIMIT_*` env vars

## Testing

```bash
cd backend
cargo test
```

## Tokenized Assets (NGX Equities & ETFs)

The backend supports listing, viewing, and selecting NGX (Nigerian Exchange) tokenized assets:

1. **List**: `GET /stocks/list?type=equity|etf|all&search=GTCO&limit=20&offset=0`
2. **View**: `GET /stocks/view/GTCO` or `GET /stocks/view/{uuid}`
3. **Choose**: `POST /stocks/choose` with `{"ngx_symbol": "GTCO"}` or `{"asset_id": "uuid"}`

Place orders using the chosen asset's `ngx_symbol` as `stock_symbol` in `/trade/place-order`.

Seed data includes sample NGX equities (DANGCEM, ACCESS, GTCO, etc.) and ETFs. Add `NGX_API_BASE_URL` and `NGX_API_TOKEN` to enrich with live prices from NGX Market Data API.

## Production Checklist

- [ ] Strong `JWT_SECRET` (32+ chars)
- [ ] HTTPS / reverse proxy (nginx)
- [ ] Database connection pool tuning
- [ ] Event listener for on-chain OrderPlaced/Staked confirmations
- [ ] Signer wallet for contract writes (relayer pattern)
- [ ] Structured logs to ELK/Datadog
