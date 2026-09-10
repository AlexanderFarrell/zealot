-- Z137: server identity, principals, personal scopes, and legacy item backfill.
-- SQLite cannot add a NOT NULL foreign-key column to an existing populated
-- table.  The backfill plus triggers below provide the same runtime invariant
-- without rebuilding every table which references item.
CREATE TABLE server (
    server_id TEXT PRIMARY KEY NOT NULL,
    display_name TEXT NOT NULL DEFAULT 'Zealot server',
    schema_migration_version INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE server_principal (
    principal_id TEXT PRIMARY KEY NOT NULL,
    server_id TEXT NOT NULL REFERENCES server(server_id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('human', 'service')),
    account_id INTEGER UNIQUE REFERENCES account(account_id) ON DELETE CASCADE,
    display_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'retired')),
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    retired_at INTEGER,
    CHECK ((kind = 'human' AND account_id IS NOT NULL) OR (kind = 'service' AND account_id IS NULL)),
    UNIQUE(server_id, account_id)
);

CREATE TABLE scope (
    scope_id TEXT PRIMARY KEY NOT NULL,
    server_id TEXT NOT NULL REFERENCES server(server_id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'archived')),
    owner_principal_id TEXT NOT NULL REFERENCES server_principal(principal_id),
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE scope_member (
    scope_id TEXT NOT NULL REFERENCES scope(scope_id) ON DELETE CASCADE,
    principal_id TEXT NOT NULL REFERENCES server_principal(principal_id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('owner', 'editor', 'viewer')),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'revoked')),
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    PRIMARY KEY (scope_id, principal_id)
);

CREATE TRIGGER account_create_personal_scope AFTER INSERT ON account BEGIN
  INSERT INTO server_principal (principal_id, server_id, kind, account_id, display_name)
  SELECT lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6))), server_id, 'human', NEW.account_id, NEW.username FROM server LIMIT 1;
  INSERT INTO scope (scope_id, server_id, title, owner_principal_id)
  SELECT lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6))), s.server_id, 'Personal — ' || NEW.username, p.principal_id FROM server s JOIN server_principal p ON p.account_id = NEW.account_id;
  INSERT INTO scope_member (scope_id, principal_id, role)
  SELECT sc.scope_id, sc.owner_principal_id, 'owner' FROM scope sc JOIN server_principal p ON p.principal_id = sc.owner_principal_id WHERE p.account_id = NEW.account_id;
END;

INSERT INTO server (server_id, schema_migration_version)
SELECT lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6))), 16
WHERE NOT EXISTS (SELECT 1 FROM server);

INSERT INTO server_principal (principal_id, server_id, kind, account_id, display_name)
SELECT lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6))),
       (SELECT server_id FROM server LIMIT 1), 'human', a.account_id, a.username
FROM account a
WHERE NOT EXISTS (SELECT 1 FROM server_principal p WHERE p.account_id = a.account_id);

CREATE TABLE scope_seed (account_id INTEGER PRIMARY KEY, scope_id TEXT NOT NULL);
INSERT INTO scope_seed (account_id, scope_id)
SELECT a.account_id, lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6)))
FROM account a;

INSERT INTO scope (scope_id, server_id, title, owner_principal_id)
SELECT ss.scope_id, s.server_id, 'Personal — ' || a.username, p.principal_id
FROM scope_seed ss
JOIN account a ON a.account_id = ss.account_id
JOIN server s
JOIN server_principal p ON p.account_id = a.account_id
WHERE NOT EXISTS (SELECT 1 FROM scope sc WHERE sc.owner_principal_id = p.principal_id AND sc.title = 'Personal — ' || a.username);

INSERT INTO scope_member (scope_id, principal_id, role)
SELECT sc.scope_id, p.principal_id, 'owner'
FROM scope sc JOIN server_principal p ON p.principal_id = sc.owner_principal_id
WHERE NOT EXISTS (SELECT 1 FROM scope_member sm WHERE sm.scope_id = sc.scope_id AND sm.principal_id = p.principal_id);

ALTER TABLE item ADD COLUMN scope_id TEXT REFERENCES scope(scope_id);
UPDATE item
SET scope_id = (
  SELECT sc.scope_id FROM scope sc
  JOIN server_principal p ON p.principal_id = sc.owner_principal_id
  WHERE p.account_id = item.account_id
)
WHERE scope_id IS NULL;
CREATE INDEX idx_item_scope_item ON item(scope_id, item_id);
CREATE TRIGGER item_scope_required_update BEFORE UPDATE OF scope_id ON item
WHEN NEW.scope_id IS NULL BEGIN SELECT RAISE(ABORT, 'item.scope_id is required'); END;
DROP TABLE scope_seed;
