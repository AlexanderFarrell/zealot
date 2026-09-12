-- Z139: SQLite needs triggers to enforce the non-null rule scope invariant.
CREATE TRIGGER rule_scope_required_insert BEFORE INSERT ON rule
WHEN NEW.scope_id IS NULL BEGIN
    SELECT RAISE(ABORT, 'rule.scope_id is required');
END;

CREATE TRIGGER rule_scope_required_update BEFORE UPDATE OF scope_id ON rule
WHEN NEW.scope_id IS NULL BEGIN
    SELECT RAISE(ABORT, 'rule.scope_id is required');
END;
