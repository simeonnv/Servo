-- Add migration script here
CREATE TABLE IF NOT EXISTS Accounts (
    account_id UUID PRIMARY KEY,
    username VARCHAR(64) NOT NULL UNIQUE,
    password VARCHAR(256) NOT NULL,
    role VARCHAR(32) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_accounts_account_id ON Accounts (account_id);

CREATE INDEX IF NOT EXISTS idx_accounts_username ON Accounts (username);