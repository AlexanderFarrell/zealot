-- Z139: automation rules are owned by a scope, not only by an account.
ALTER TABLE rule ADD COLUMN scope_id TEXT REFERENCES scope(scope_id);

UPDATE rule
SET scope_id = (
    SELECT p.default_scope_id
    FROM server_principal p
    WHERE p.account_id = rule.account_id
      AND p.kind = 'human'
      AND p.default_scope_id IS NOT NULL
)
WHERE scope_id IS NULL;

CREATE INDEX idx_rule_scope ON rule(scope_id);
CREATE INDEX idx_rule_scope_event_rules ON rule(scope_id, enabled, trigger_kind);
