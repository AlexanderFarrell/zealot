-- Z139: SQLite needs a table rebuild to make account ownership optional.
CREATE TABLE api_key_new (
    api_key_id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER REFERENCES account(account_id) ON DELETE CASCADE,
    principal_id TEXT NOT NULL REFERENCES server_principal(principal_id),
    key_hash TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL DEFAULT 'Default',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
INSERT INTO api_key_new (api_key_id, account_id, principal_id, key_hash, label, created_at)
SELECT k.api_key_id, k.account_id, p.principal_id, k.key_hash, k.label, k.created_at
FROM api_key k JOIN server_principal p ON p.account_id = k.account_id AND p.kind = 'human';
DROP TABLE api_key;
ALTER TABLE api_key_new RENAME TO api_key;
CREATE INDEX idx_api_key_principal ON api_key(principal_id);

ALTER TABLE item ADD COLUMN created_by_principal_id TEXT REFERENCES server_principal(principal_id);
ALTER TABLE item ADD COLUMN updated_by_principal_id TEXT REFERENCES server_principal(principal_id);
UPDATE item
SET created_by_principal_id = (SELECT principal_id FROM server_principal p WHERE p.account_id = item.account_id AND p.kind = 'human'),
    updated_by_principal_id = (SELECT principal_id FROM server_principal p WHERE p.account_id = item.account_id AND p.kind = 'human');
