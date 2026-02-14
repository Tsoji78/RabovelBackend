-- Tokenized assets: NGX equities and ETFs with on-chain token mapping
CREATE TABLE IF NOT EXISTS tokenized_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ngx_symbol VARCHAR(20) NOT NULL,
    name VARCHAR(255) NOT NULL,
    asset_type VARCHAR(20) NOT NULL CHECK (asset_type IN ('equity', 'etf')),
    token_address VARCHAR(42),
    description TEXT,
    last_price DECIMAL(36, 18),
    change_pct DECIMAL(18, 6),
    volume BIGINT,
    market_cap DECIMAL(36, 18),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(ngx_symbol, asset_type)
);

CREATE INDEX idx_tokenized_assets_type ON tokenized_assets(asset_type);
CREATE INDEX idx_tokenized_assets_ngx_symbol ON tokenized_assets(ngx_symbol);
CREATE INDEX idx_tokenized_assets_token ON tokenized_assets(token_address) WHERE token_address IS NOT NULL;
