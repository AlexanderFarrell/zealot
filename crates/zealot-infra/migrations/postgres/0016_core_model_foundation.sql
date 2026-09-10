-- Z137: server identity, principals, personal scopes, and legacy item backfill.
CREATE TABLE server (
    server_id uuid PRIMARY KEY,
    display_name text NOT NULL DEFAULT 'Zealot server',
    schema_migration_version bigint NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

INSERT INTO server (server_id, schema_migration_version)
SELECT md5(random()::text || clock_timestamp()::text)::uuid, 16
WHERE NOT EXISTS (SELECT 1 FROM server);

CREATE TABLE server_principal (
    principal_id uuid PRIMARY KEY,
    server_id uuid NOT NULL REFERENCES server(server_id) ON DELETE CASCADE,
    kind text NOT NULL CHECK (kind IN ('human', 'service')),
    account_id integer UNIQUE REFERENCES account(account_id) ON DELETE CASCADE,
    display_name text NOT NULL,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'retired')),
    created_at timestamptz NOT NULL DEFAULT now(),
    retired_at timestamptz,
    CHECK ((kind = 'human' AND account_id IS NOT NULL) OR (kind = 'service' AND account_id IS NULL)),
    UNIQUE(server_id, account_id)
);
CREATE TABLE scope (
    scope_id uuid PRIMARY KEY,
    server_id uuid NOT NULL REFERENCES server(server_id) ON DELETE CASCADE,
    title text NOT NULL,
    description text,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'archived')),
    owner_principal_id uuid NOT NULL REFERENCES server_principal(principal_id),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE scope_member (
    scope_id uuid NOT NULL REFERENCES scope(scope_id) ON DELETE CASCADE,
    principal_id uuid NOT NULL REFERENCES server_principal(principal_id) ON DELETE CASCADE,
    role text NOT NULL CHECK (role IN ('owner', 'editor', 'viewer')),
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'revoked')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (scope_id, principal_id)
);

CREATE FUNCTION create_personal_scope_for_account() RETURNS trigger AS $$
DECLARE principal uuid;
DECLARE personal_scope uuid;
BEGIN
  INSERT INTO server_principal (principal_id, server_id, kind, account_id, display_name)
  SELECT md5(random()::text || NEW.account_id::text || clock_timestamp()::text)::uuid, server_id, 'human', NEW.account_id, NEW.username FROM server LIMIT 1 RETURNING principal_id INTO principal;
  INSERT INTO scope (scope_id, server_id, title, owner_principal_id)
  SELECT md5(random()::text || NEW.account_id::text || clock_timestamp()::text)::uuid, server_id, 'Personal — ' || NEW.username, principal FROM server LIMIT 1 RETURNING scope_id INTO personal_scope;
  INSERT INTO scope_member (scope_id, principal_id, role) VALUES (personal_scope, principal, 'owner');
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER account_create_personal_scope AFTER INSERT ON account FOR EACH ROW EXECUTE FUNCTION create_personal_scope_for_account();

INSERT INTO server_principal (principal_id, server_id, kind, account_id, display_name)
SELECT md5(random()::text || a.account_id::text || clock_timestamp()::text)::uuid,
       (SELECT server_id FROM server LIMIT 1), 'human', a.account_id, a.username
FROM account a
WHERE NOT EXISTS (SELECT 1 FROM server_principal p WHERE p.account_id = a.account_id);

INSERT INTO scope (scope_id, server_id, title, owner_principal_id)
SELECT md5(random()::text || a.account_id::text || clock_timestamp()::text)::uuid,
       s.server_id, 'Personal — ' || a.username, p.principal_id
FROM account a CROSS JOIN server s JOIN server_principal p ON p.account_id = a.account_id
WHERE NOT EXISTS (SELECT 1 FROM scope sc WHERE sc.owner_principal_id = p.principal_id);
INSERT INTO scope_member (scope_id, principal_id, role)
SELECT sc.scope_id, sc.owner_principal_id, 'owner' FROM scope sc
ON CONFLICT (scope_id, principal_id) DO NOTHING;

ALTER TABLE item ADD COLUMN scope_id uuid;
UPDATE item i SET scope_id = sc.scope_id
FROM scope sc JOIN server_principal p ON p.principal_id = sc.owner_principal_id
WHERE p.account_id = i.account_id;
ALTER TABLE item ALTER COLUMN scope_id SET NOT NULL;
ALTER TABLE item ADD CONSTRAINT item_scope_id_fkey FOREIGN KEY (scope_id) REFERENCES scope(scope_id);
CREATE INDEX idx_item_scope_item ON item(scope_id, item_id);
