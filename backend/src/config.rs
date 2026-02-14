//! Application configuration loaded from environment variables.

use std::env;

/// Application configuration - all values from env vars for 12-factor compliance.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub rpc_url: String,
    pub chain_id: u64,
    pub platform_token_address: Option<String>,
    pub staking_contract_address: Option<String>,
    pub trading_contract_address: Option<String>,
    pub ngx_api_base_url: Option<String>,
    pub ngx_api_token: Option<String>,
    pub server_host: String,
    pub server_port: u16,
    pub cors_origins: Vec<String>,
    pub rate_limit_requests: u32,
    pub rate_limit_window_secs: u64,
}

impl Config {
    /// Load configuration from environment. Panics if required vars are missing.
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")?;
        let jwt_secret = env::var("JWT_SECRET")?;
        let jwt_expiry_hours = env::var("JWT_EXPIRY_HOURS")
            .unwrap_or_else(|_| "24".into())
            .parse()
            .unwrap_or(24);
        let rpc_url = env::var("RPC_URL")?;
        let chain_id = env::var("CHAIN_ID")
            .unwrap_or_else(|_| "11155111".into())
            .parse()
            .unwrap_or(11155111); // Sepolia default

        let platform_token_address = env::var("PLATFORM_TOKEN_ADDRESS").ok();
        let staking_contract_address = env::var("STAKING_CONTRACT_ADDRESS").ok();
        let trading_contract_address = env::var("TRADING_CONTRACT_ADDRESS").ok();
        let ngx_api_base_url = env::var("NGX_API_BASE_URL").ok();
        let ngx_api_token = env::var("NGX_API_TOKEN").ok();

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".into())
            .parse()
            .unwrap_or(8080);

        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "*".into())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let rate_limit_requests = env::var("RATE_LIMIT_REQUESTS")
            .unwrap_or_else(|_| "100".into())
            .parse()
            .unwrap_or(100);
        let rate_limit_window_secs = env::var("RATE_LIMIT_WINDOW_SECS")
            .unwrap_or_else(|_| "60".into())
            .parse()
            .unwrap_or(60);

        Ok(Self {
            database_url,
            jwt_secret,
            jwt_expiry_hours,
            rpc_url,
            chain_id,
            platform_token_address,
            staking_contract_address,
            trading_contract_address,
            ngx_api_base_url,
            ngx_api_token,
            server_host,
            server_port,
            cors_origins,
            rate_limit_requests,
            rate_limit_window_secs,
        })
    }
}
