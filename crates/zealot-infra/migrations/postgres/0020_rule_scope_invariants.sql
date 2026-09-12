-- Z139: PostgreSQL 0019 already enforces this invariant with NOT NULL and FK.
-- Validate the immutable foreign-key constraint as the paired follow-up.
ALTER TABLE rule VALIDATE CONSTRAINT rule_scope_id_fkey;
