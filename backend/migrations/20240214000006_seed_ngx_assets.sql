-- Seed sample NGX equities and ETFs (tokenized or available for tokenization)
-- Replace with actual NGX symbols; add token_address when on-chain tokens exist
INSERT INTO tokenized_assets (ngx_symbol, name, asset_type, description, created_at, updated_at)
VALUES
    ('DANGCEM', 'Dangote Cement Plc', 'equity', 'Nigerian cement manufacturer', NOW(), NOW()),
    ('ACCESS', 'Access Holdings Plc', 'equity', 'Financial services group', NOW(), NOW()),
    ('GTCO', 'Guaranty Trust Holding Company Plc', 'equity', 'Banking and financial services', NOW(), NOW()),
    ('ZENITHBANK', 'Zenith Bank Plc', 'equity', 'Commercial bank', NOW(), NOW()),
    ('NB', 'Nigerian Breweries Plc', 'equity', 'Beverage company', NOW(), NOW()),
    ('MTNN', 'MTN Nigeria Communications Plc', 'equity', 'Telecommunications', NOW(), NOW()),
    ('NGX30ETF', 'Vetiva NGX 30 ETF', 'etf', 'Tracks NGX 30 Index', NOW(), NOW()),
    ('VETINDETF', 'Vetiva Industrial Goods ETF', 'etf', 'Industrial goods sector ETF', NOW(), NOW()),
    ('STANBICETF', 'Stanbic IBTC ETF', 'etf', 'Equity ETF', NOW(), NOW())
ON CONFLICT (ngx_symbol, asset_type) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    updated_at = NOW();
