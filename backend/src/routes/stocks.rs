//! Tokenized assets routes - list, view, choose NGX equities and ETFs.

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::config::Config;
use crate::error::AppError;

/// Asset type filter: equity, etf, or all
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetTypeFilter {
    Equity,
    Etf,
    All,
}

impl Default for AssetTypeFilter {
    fn default() -> Self {
        AssetTypeFilter::All
    }
}

impl std::str::FromStr for AssetTypeFilter {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "equity" | "equities" => Ok(AssetTypeFilter::Equity),
            "etf" | "etfs" => Ok(AssetTypeFilter::Etf),
            "all" | "" => Ok(AssetTypeFilter::All),
            _ => Err(format!("Invalid type: {}", s)),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct ListQuery {
    #[validate(length(max = 20))]
    pub r#type: Option<String>,
    #[validate(length(max = 50))]
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct TokenizedAssetItem {
    pub id: String,
    pub ngx_symbol: String,
    pub name: String,
    pub asset_type: String,
    pub token_address: Option<String>,
    pub last_price: Option<String>,
    pub change_pct: Option<String>,
    pub volume: Option<i64>,
    pub market_cap: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TokenizedAssetsResponse {
    pub assets: Vec<TokenizedAssetItem>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct TokenizedAssetDetail {
    pub id: String,
    pub ngx_symbol: String,
    pub name: String,
    pub asset_type: String,
    pub token_address: Option<String>,
    pub description: Option<String>,
    pub last_price: Option<String>,
    pub change_pct: Option<String>,
    pub volume: Option<i64>,
    pub market_cap: Option<String>,
}

/// List tokenized assets (NGX equities and ETFs) with optional filters.
/// GET /stocks/list?type=equity|etf|all&search=ACCESS&limit=20&offset=0
pub async fn list(
    pool: web::Data<PgPool>,
    _config: web::Data<Config>,
    query: web::Query<ListQuery>,
) -> Result<HttpResponse, AppError> {
    query.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let asset_type = query
        .r#type
        .as_deref()
        .unwrap_or("all")
        .parse()
        .map_err(|e: String| AppError::Validation(e))?;

    let search = query.search.as_deref().unwrap_or("").trim();
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    let search_pattern = format!("%{}%", search);

    let type_clause: &str = match asset_type {
        AssetTypeFilter::Equity => "AND asset_type = 'equity'",
        AssetTypeFilter::Etf => "AND asset_type = 'etf'",
        AssetTypeFilter::All => "",
    };

    let (total, rows) = if search.is_empty() {
        let count_sql = format!(
            r#"SELECT COUNT(*)::bigint FROM tokenized_assets WHERE 1=1 {}"#,
            type_clause
        );
        let total: (i64,) = sqlx::query_as(&count_sql)
            .fetch_one(pool.get_ref())
            .await?;

        let list_sql = format!(
            r#"SELECT id, ngx_symbol, name, asset_type, token_address, last_price, change_pct, volume, market_cap
               FROM tokenized_assets WHERE 1=1 {}
               ORDER BY asset_type, name
               LIMIT $1 OFFSET $2"#,
            type_clause
        );
        let rows = sqlx::query_as::<_, TokenizedAssetRow>(&list_sql)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool.get_ref())
            .await?;

        (total.0, rows)
    } else {
        let count_sql = format!(
            r#"SELECT COUNT(*)::bigint FROM tokenized_assets
               WHERE (ngx_symbol ILIKE $1 OR name ILIKE $1) {}"#,
            type_clause
        );
        let total: (i64,) = sqlx::query_as(&count_sql)
            .bind(&search_pattern)
            .fetch_one(pool.get_ref())
            .await?;

        let list_sql = format!(
            r#"SELECT id, ngx_symbol, name, asset_type, token_address, last_price, change_pct, volume, market_cap
               FROM tokenized_assets
               WHERE (ngx_symbol ILIKE $1 OR name ILIKE $1) {}
               ORDER BY name LIMIT $2 OFFSET $3"#,
            type_clause
        );
        let rows = sqlx::query_as::<_, TokenizedAssetRow>(&list_sql)
            .bind(&search_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool.get_ref())
            .await?;

        (total.0, rows)
    };

    let assets: Vec<TokenizedAssetItem> = rows
        .into_iter()
        .map(|r| TokenizedAssetItem {
            id: r.id.to_string(),
            ngx_symbol: r.ngx_symbol,
            name: r.name,
            asset_type: r.asset_type,
            token_address: r.token_address,
            last_price: r.last_price.map(|d| d.to_string()),
            change_pct: r.change_pct.map(|d| d.to_string()),
            volume: r.volume,
            market_cap: r.market_cap.map(|d| d.to_string()),
        })
        .collect();

    Ok(HttpResponse::Ok().json(TokenizedAssetsResponse {
        assets,
        total,
    }))
}

/// View single tokenized asset by ID or NGX symbol.
/// GET /stocks/view/{id_or_symbol}
pub async fn view(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let id_or_symbol = path.into_inner();

    let row = if Uuid::parse_str(&id_or_symbol).is_ok() {
        sqlx::query_as::<_, TokenizedAssetDetailRow>(
            r#"SELECT id, ngx_symbol, name, asset_type, token_address, description, last_price, change_pct, volume, market_cap
               FROM tokenized_assets WHERE id = $1"#,
        )
        .bind(Uuid::parse_str(&id_or_symbol).unwrap())
        .fetch_optional(pool.get_ref())
        .await?
    } else {
        sqlx::query_as::<_, TokenizedAssetDetailRow>(
            r#"SELECT id, ngx_symbol, name, asset_type, token_address, description, last_price, change_pct, volume, market_cap
               FROM tokenized_assets WHERE ngx_symbol = $1 LIMIT 1"#,
        )
        .bind(&id_or_symbol)
        .fetch_optional(pool.get_ref())
        .await?
    };

    let r = row.ok_or_else(|| AppError::NotFound("Asset not found".to_string()))?;

    Ok(HttpResponse::Ok().json(TokenizedAssetDetail {
        id: r.id.to_string(),
        ngx_symbol: r.ngx_symbol,
        name: r.name,
        asset_type: r.asset_type,
        token_address: r.token_address,
        description: r.description,
        last_price: r.last_price.map(|d| d.to_string()),
        change_pct: r.change_pct.map(|d| d.to_string()),
        volume: r.volume,
        market_cap: r.market_cap.map(|d| d.to_string()),
    }))
}

/// Choose/select asset for trading - returns full detail for the chosen asset.
/// POST /stocks/choose body: { "asset_id" or "ngx_symbol" }
/// Use by client to confirm selection before placing orders.
#[derive(Debug, Deserialize)]
pub struct ChooseRequest {
    pub asset_id: Option<String>,
    pub ngx_symbol: Option<String>,
}

pub async fn choose(
    pool: web::Data<PgPool>,
    body: web::Json<ChooseRequest>,
) -> Result<HttpResponse, AppError> {
    let row = if let Some(ref id) = body.asset_id {
        if let Ok(uuid) = Uuid::parse_str(id) {
            sqlx::query_as::<_, TokenizedAssetDetailRow>(
                r#"SELECT id, ngx_symbol, name, asset_type, token_address, description, last_price, change_pct, volume, market_cap
                   FROM tokenized_assets WHERE id = $1"#,
            )
            .bind(uuid)
            .fetch_optional(pool.get_ref())
            .await?
        } else {
            None
        }
    } else if let Some(ref sym) = body.ngx_symbol {
        sqlx::query_as::<_, TokenizedAssetDetailRow>(
            r#"SELECT id, ngx_symbol, name, asset_type, token_address, description, last_price, change_pct, volume, market_cap
               FROM tokenized_assets WHERE ngx_symbol = $1 LIMIT 1"#,
        )
        .bind(sym)
        .fetch_optional(pool.get_ref())
        .await?
    } else {
        return Err(AppError::Validation(
            "Provide asset_id or ngx_symbol".to_string(),
        ));
    };

    let r = row.ok_or_else(|| AppError::NotFound("Asset not found".to_string()))?;

    Ok(HttpResponse::Ok().json(TokenizedAssetDetail {
        id: r.id.to_string(),
        ngx_symbol: r.ngx_symbol,
        name: r.name,
        asset_type: r.asset_type,
        token_address: r.token_address,
        description: r.description,
        last_price: r.last_price.map(|d| d.to_string()),
        change_pct: r.change_pct.map(|d| d.to_string()),
        volume: r.volume,
        market_cap: r.market_cap.map(|d| d.to_string()),
    }))
}

#[derive(Debug, sqlx::FromRow)]
struct TokenizedAssetRow {
    id: Uuid,
    ngx_symbol: String,
    name: String,
    asset_type: String,
    token_address: Option<String>,
    last_price: Option<rust_decimal::Decimal>,
    change_pct: Option<rust_decimal::Decimal>,
    volume: Option<i64>,
    market_cap: Option<rust_decimal::Decimal>,
}

#[derive(Debug, sqlx::FromRow)]
struct TokenizedAssetDetailRow {
    id: Uuid,
    ngx_symbol: String,
    name: String,
    asset_type: String,
    token_address: Option<String>,
    description: Option<String>,
    last_price: Option<rust_decimal::Decimal>,
    change_pct: Option<rust_decimal::Decimal>,
    volume: Option<i64>,
    market_cap: Option<rust_decimal::Decimal>,
}
