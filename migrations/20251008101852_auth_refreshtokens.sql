CREATE TABLE IF NOT EXISTS RefreshTokens (
    refresh_token_id UUID PRIMARY KEY,
    account_id UUID NOT NULL,
    refresh_token VARCHAR(256) NOT NULL UNIQUE,
    role VARCHAR(16) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES Accounts (account_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_refresh_tokens_refresh_token_id ON RefreshTokens (refresh_token_id);