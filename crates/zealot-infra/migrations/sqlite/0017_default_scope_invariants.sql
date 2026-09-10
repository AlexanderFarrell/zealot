-- Z138: make a human principal's personal scope explicit and enforce the
-- item-scope invariant at insert time.  0016 may already be deployed.
ALTER TABLE server_principal
    ADD COLUMN default_scope_id TEXT REFERENCES scope(scope_id);

UPDATE server_principal
SET default_scope_id = (
    SELECT scope_id
    FROM scope
    WHERE owner_principal_id = server_principal.principal_id
      AND status = 'active'
    ORDER BY created_at, scope_id
    LIMIT 1
)
WHERE kind = 'human' AND default_scope_id IS NULL;

DROP TRIGGER account_create_personal_scope;
CREATE TRIGGER account_create_personal_scope AFTER INSERT ON account BEGIN
  INSERT INTO server_principal (principal_id, server_id, kind, account_id, display_name)
  SELECT lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6))), server_id, 'human', NEW.account_id, NEW.username FROM server LIMIT 1;
  INSERT INTO scope (scope_id, server_id, title, owner_principal_id)
  SELECT lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6))), s.server_id, 'Personal — ' || NEW.username, p.principal_id FROM server s JOIN server_principal p ON p.account_id = NEW.account_id;
  UPDATE server_principal
  SET default_scope_id = (
    SELECT scope_id FROM scope WHERE owner_principal_id = server_principal.principal_id ORDER BY created_at, scope_id LIMIT 1
  )
  WHERE account_id = NEW.account_id;
  INSERT INTO scope_member (scope_id, principal_id, role)
  SELECT p.default_scope_id, p.principal_id, 'owner' FROM server_principal p WHERE p.account_id = NEW.account_id;
END;

CREATE TRIGGER item_scope_required_insert BEFORE INSERT ON item
WHEN NEW.scope_id IS NULL BEGIN SELECT RAISE(ABORT, 'item.scope_id is required'); END;
