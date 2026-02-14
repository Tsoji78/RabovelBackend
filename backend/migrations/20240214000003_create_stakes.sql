-- Stakes table for staking positions
CREATE TABLE stakes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    amount DECIMAL(36, 18) NOT NULL CHECK (amount > 0),
    locked_until TIMESTAMPTZ NOT NULL,
    reward_claimed DECIMAL(36, 18) NOT NULL DEFAULT 0,
    tx_hash VARCHAR(66),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_stakes_user ON stakes(user_id);
CREATE INDEX idx_stakes_locked ON stakes(locked_until);
