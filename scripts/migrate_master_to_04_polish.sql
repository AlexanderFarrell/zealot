-- migrate_master_to_04_polish.sql
--
-- Brings a PostgreSQL database running the Zealot `master` schema up to the
-- state produced by the four migrations in the `04-polish` branch.
--
-- Safe to re-run: all DDL steps guard against already-applied changes.
-- Run as a transaction so the whole script rolls back on any error.
--
-- Usage:
--   psql -d <your_zealot_db> -f scripts/migrate_master_to_04_polish.sql

BEGIN;

-- ============================================================
-- 1. Split account.full_name → given_name + surname
--    (mirrors 0002_account_split_full_name.sql)
-- ============================================================

DO $$
BEGIN
    -- Only run if full_name still exists and given_name does not
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'account' AND column_name = 'full_name'
    ) AND NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'account' AND column_name = 'given_name'
    ) THEN
        ALTER TABLE account ADD COLUMN given_name varchar(100) NOT NULL DEFAULT '';
        ALTER TABLE account ADD COLUMN surname    varchar(100) NOT NULL DEFAULT '';

        UPDATE account SET
            given_name = CASE
                WHEN full_name LIKE '% %' THEN split_part(full_name, ' ', 1)
                ELSE full_name
            END,
            surname = CASE
                WHEN full_name LIKE '% %'
                    THEN substring(full_name FROM position(' ' IN full_name) + 1)
                ELSE ''
            END;

        ALTER TABLE account DROP COLUMN full_name;
    END IF;
END;
$$;

-- ============================================================
-- 2. Create item_relationship enum type
-- ============================================================

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_type WHERE typname = 'item_relationship'
    ) THEN
        CREATE TYPE item_relationship AS ENUM (
            'Parent',
            'Blocks',
            'Tag',
            'Topic',
            'Other'
        );
    END IF;
END;
$$;

-- ============================================================
-- 3. Create item_item_link table
-- ============================================================

CREATE TABLE IF NOT EXISTS item_item_link (
    item_item_id   serial  PRIMARY KEY,
    first_item_id  int     NOT NULL REFERENCES item(item_id) ON DELETE CASCADE,
    second_item_id int     NOT NULL REFERENCES item(item_id) ON DELETE CASCADE,
    relationship   item_relationship NOT NULL DEFAULT 'Parent',
    CONSTRAINT u_item_link UNIQUE (first_item_id, second_item_id)
);

-- ============================================================
-- 4. Backfill Parent links from attribute + attribute_list_value
--    (mirrors 0003_backfill_parent_links.sql)
-- ============================================================

-- From attribute.value_item_id (direct item reference)
INSERT INTO item_item_link (first_item_id, second_item_id, relationship)
SELECT child.item_id, parent.item_id, 'Parent'
FROM item child
JOIN attribute child_attr ON child_attr.item_id = child.item_id
JOIN item parent ON parent.item_id = child_attr.value_item_id
WHERE child_attr.key = 'Parent'
  AND child_attr.value_item_id IS NOT NULL
  AND parent.account_id = child.account_id
ON CONFLICT DO NOTHING;

-- From attribute_list_value.value_item_id (list of item references)
INSERT INTO item_item_link (first_item_id, second_item_id, relationship)
SELECT child.item_id, parent.item_id, 'Parent'
FROM item child
JOIN attribute_list_value child_attr ON child_attr.item_id = child.item_id
JOIN item parent ON parent.item_id = child_attr.value_item_id
WHERE child_attr.key = 'Parent'
  AND child_attr.value_item_id IS NOT NULL
  AND parent.account_id = child.account_id
ON CONFLICT DO NOTHING;

-- From attribute.value_text (title-based reference)
INSERT INTO item_item_link (first_item_id, second_item_id, relationship)
SELECT child.item_id, parent.item_id, 'Parent'
FROM item child
JOIN attribute child_attr ON child_attr.item_id = child.item_id
JOIN item parent ON parent.account_id = child.account_id
    AND parent.title = child_attr.value_text
WHERE child_attr.key = 'Parent'
  AND child_attr.value_text IS NOT NULL
ON CONFLICT DO NOTHING;

-- From attribute_list_value.value_text (list of title-based references)
INSERT INTO item_item_link (first_item_id, second_item_id, relationship)
SELECT child.item_id, parent.item_id, 'Parent'
FROM item child
JOIN attribute_list_value child_attr ON child_attr.item_id = child.item_id
JOIN item parent ON parent.account_id = child.account_id
    AND parent.title = child_attr.value_text
WHERE child_attr.key = 'Parent'
  AND child_attr.value_text IS NOT NULL
ON CONFLICT DO NOTHING;

-- ============================================================
-- 5. Create session table
-- ============================================================

CREATE TABLE IF NOT EXISTS session (
    token_hash  text        PRIMARY KEY,
    account_id  bigint      NOT NULL REFERENCES account(account_id) ON DELETE CASCADE,
    expires_at  timestamptz NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_session_account_id ON session(account_id);

-- ============================================================
-- 6. Create rule table + indexes
--    (mirrors 0004_rules.sql)
-- ============================================================

CREATE TABLE IF NOT EXISTS rule (
    rule_id        bigserial   PRIMARY KEY,
    account_id     bigint      NOT NULL REFERENCES account(account_id) ON DELETE CASCADE,
    name           text        NOT NULL,
    description    text        NOT NULL DEFAULT '',
    trigger_kind   text        NOT NULL,
    trigger_config text        NOT NULL DEFAULT '{}',
    script         text        NOT NULL DEFAULT '',
    enabled        boolean     NOT NULL DEFAULT true,
    created_at     timestamptz NOT NULL,
    last_run_at    timestamptz,
    last_error     text,
    last_output    text
);

CREATE INDEX IF NOT EXISTS idx_rule_account     ON rule(account_id);
CREATE INDEX IF NOT EXISTS idx_rule_event_rules ON rule(enabled, trigger_kind);

-- ============================================================
-- 7. Insert missing system attribute_kinds
--    (the 5 that master did not seed)
-- ============================================================

INSERT INTO attribute_kind (key, description, base_type, is_system, config)
SELECT 'Time of Day', 'The general time of day which something occurred.', 'dropdown', true,
    '{"values": ["Morning", "Afternoon", "Evening", "Anytime"]}'
WHERE NOT EXISTS (SELECT 1 FROM attribute_kind WHERE key = 'Time of Day' AND is_system);

INSERT INTO attribute_kind (key, description, base_type, is_system, config)
SELECT 'Phone', 'International phone number (E.164).', 'text', true,
    '{"pattern": "[+][1-9][0-9]{1,14}"}'
WHERE NOT EXISTS (SELECT 1 FROM attribute_kind WHERE key = 'Phone' AND is_system);

INSERT INTO attribute_kind (key, description, base_type, is_system, config)
SELECT 'Email', 'Email address.', 'text', true,
    '{"pattern": "[^@[:space:]]+@[^@[:space:]]+[.][^@[:space:]]+"}'
WHERE NOT EXISTS (SELECT 1 FROM attribute_kind WHERE key = 'Email' AND is_system);

INSERT INTO attribute_kind (key, description, base_type, is_system, config)
SELECT 'Schedule', 'Configuration for how often something should happen', 'text', true,
    '{"pattern": "[0-1]{10}"}'
WHERE NOT EXISTS (SELECT 1 FROM attribute_kind WHERE key = 'Schedule' AND is_system);

INSERT INTO attribute_kind (key, description, base_type, is_system, config)
SELECT 'End Date', 'The date at which something stops.', 'date', true, '{}'
WHERE NOT EXISTS (SELECT 1 FROM attribute_kind WHERE key = 'End Date' AND is_system);

-- ============================================================
-- 8. Drop api_key table
--    The 04-polish branch uses session tokens instead of API keys.
--    Any existing API keys will be invalidated.
-- ============================================================

DROP TABLE IF EXISTS api_key;

COMMIT;
