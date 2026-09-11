-- Z139: API keys authenticate principals, including non-human service users.
ALTER TABLE api_key ADD COLUMN principal_id uuid REFERENCES server_principal(principal_id);

UPDATE api_key k
SET principal_id = p.principal_id
FROM server_principal p
WHERE p.account_id = k.account_id
  AND p.kind = 'human'
  AND k.principal_id IS NULL;

ALTER TABLE api_key ALTER COLUMN account_id DROP NOT NULL;
ALTER TABLE api_key ALTER COLUMN principal_id SET NOT NULL;
ALTER TABLE api_key ADD CONSTRAINT api_key_identity_check CHECK (
    (account_id IS NOT NULL AND principal_id IS NOT NULL)
    OR (account_id IS NULL AND principal_id IS NOT NULL)
);
CREATE INDEX idx_api_key_principal ON api_key(principal_id);

CREATE FUNCTION bind_human_api_key_principal() RETURNS trigger AS $$
BEGIN
  IF NEW.principal_id IS NULL AND NEW.account_id IS NOT NULL THEN
    SELECT principal_id INTO NEW.principal_id FROM server_principal
    WHERE account_id = NEW.account_id AND kind = 'human';
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER api_key_bind_human_principal BEFORE INSERT ON api_key
FOR EACH ROW EXECUTE FUNCTION bind_human_api_key_principal();

-- Record the actual actor independently of legacy account ownership.  These
-- fields are nullable for rows created before scope authorization existed.
ALTER TABLE item ADD COLUMN created_by_principal_id uuid REFERENCES server_principal(principal_id);
ALTER TABLE item ADD COLUMN updated_by_principal_id uuid REFERENCES server_principal(principal_id);
UPDATE item i SET created_by_principal_id = p.principal_id,
                  updated_by_principal_id = p.principal_id
FROM server_principal p
WHERE p.account_id = i.account_id AND p.kind = 'human';
