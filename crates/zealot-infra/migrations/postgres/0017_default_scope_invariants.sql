-- Z138: make a human principal's personal scope explicit.  0016 may already
-- be deployed, so its bootstrap function is replaced in this follow-up.
ALTER TABLE server_principal
    ADD COLUMN default_scope_id uuid REFERENCES scope(scope_id);

UPDATE server_principal p
SET default_scope_id = (
    SELECT sc.scope_id
    FROM scope
    WHERE scope.owner_principal_id = p.principal_id
      AND scope.status = 'active'
    ORDER BY scope.created_at, scope.scope_id
    LIMIT 1
)
WHERE p.kind = 'human'
  AND p.default_scope_id IS NULL;

DROP TRIGGER account_create_personal_scope ON account;
CREATE OR REPLACE FUNCTION create_personal_scope_for_account() RETURNS trigger AS $$
DECLARE principal uuid;
DECLARE personal_scope uuid;
BEGIN
  INSERT INTO server_principal (principal_id, server_id, kind, account_id, display_name)
  SELECT md5(random()::text || NEW.account_id::text || clock_timestamp()::text)::uuid, server_id, 'human', NEW.account_id, NEW.username FROM server LIMIT 1 RETURNING principal_id INTO principal;
  INSERT INTO scope (scope_id, server_id, title, owner_principal_id)
  SELECT md5(random()::text || NEW.account_id::text || clock_timestamp()::text)::uuid, server_id, 'Personal — ' || NEW.username, principal FROM server LIMIT 1 RETURNING scope_id INTO personal_scope;
  UPDATE server_principal SET default_scope_id = personal_scope WHERE principal_id = principal;
  INSERT INTO scope_member (scope_id, principal_id, role) VALUES (personal_scope, principal, 'owner');
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER account_create_personal_scope AFTER INSERT ON account FOR EACH ROW EXECUTE FUNCTION create_personal_scope_for_account();
