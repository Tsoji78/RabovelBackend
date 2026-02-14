//! NGX Market Data API v3 client.
//! Fetches equities and ETFs when NGX_API_BASE_URL and NGX_API_TOKEN are configured.
//! Docs: https://marketdataapiv3.ngxgroup.com/portal/

use serde::Deserialize;
use std::sync::Arc;
use tracing::warn;

use crate::config::Config;
use crate::error::AppError;

/// NGX API quote/structure (adapt to actual API response format)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NgxQuote {
    pub symbol: Option<String>,
    pub security_name: Option<String>,
    pub last: Option<f64>,
    pub change: Option<f64>,
    pub change_pct: Option<f64>,
    pub volume: Option<i64>,
    pub market_cap: Option<f64>,
}

/// NGX client - fetches live data when configured.
#[derive(Clone)]
pub struct NgxClient {
    client: reqwest::Client,
    base_url: String,
    token: Option<String>,
}

impl NgxClient {
    pub fn new(config: &Config) -> Option<Self> {
        let base_url = config.ngx_api_base_url.as_deref()?;
        let token = config.ngx_api_token.clone();
        if token.is_none() {
            warn!("NGX_API_BASE_URL set but NGX_API_TOKEN missing; NGX live data disabled");
        }
        Some(Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .ok()?,
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
        })
    }

    /// Fetch stock/ETF quote. Endpoint varies per NGX API version.
    pub async fn get_quote(&self, symbol: &str, product: &str) -> Result<Option<NgxQuote>, AppError> {
        let token = match &self.token {
            Some(t) => t,
            None => return Ok(None),
        };

        let url = format!(
            "{}/v3/api/{}.json?symbol={}&_t={}",
            self.base_url,
            product,
            symbol,
            token
        );

        let res = self.client.get(&url).send().await;
        match res {
            Ok(r) if r.status().is_success() => {
                let quote: NgxQuote = r.json().await.map_err(|e| {
                    AppError::Internal(anyhow::anyhow!("NGX parse error: {}", e))
                })?;
                Ok(Some(quote))
            }
            Ok(r) => {
                warn!("NGX API error: {}", r.status());
                Ok(None)
            }
            Err(e) => {
                warn!("NGX request failed: {}", e);
                Err(AppError::Internal(anyhow::anyhow!("NGX request failed: {}", e)))
            }
        }
    }
}
