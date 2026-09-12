-- Z139: automation rules are owned by a scope, not only by an account.
ALTER TABLE rule ADD COLUMN scope_id uuid;

UPDATE rule r
SET scope_id = p.default_scope_id
FROM server_principal p
WHERE p.account_id = r.account_id
  AND p.kind = 'human'
  AND p.default_scope_id IS NOT NULL;

ALTER TABLE rule ALTER COLUMN scope_id SET NOT NULL;
ALTER TABLE rule ADD CONSTRAINT rule_scope_id_fkey
    FOREIGN KEY (scope_id) REFERENCES scope(scope_id) ON DELETE CASCADE;

CREATE INDEX idx_rule_scope ON rule(scope_id);
CREATE INDEX idx_rule_scope_event_rules ON rule(scope_id, enabled, trigger_kind);
