CREATE TABLE api_key (
    api_key_id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL REFERENCES account(account_id) ON DELETE CASCADE,
    key_hash TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL DEFAULT 'Default',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO api_key (account_id, key_hash, label)
    SELECT account_id, api_key_hash, 'Default'
    FROM account WHERE api_key_hash IS NOT NULL;

ALTER TABLE account DROP COLUMN api_key_hash;
