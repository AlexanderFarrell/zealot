-- Z224: stored per-type item counts, so the type summary does not count about
-- a million type links on every request. Triggers keep the table correct on
-- every write path (assign, unassign, delete, archive, import, restore).

CREATE TABLE item_type_count (
    account_id     INTEGER NOT NULL REFERENCES account(account_id) ON DELETE CASCADE,
    type_id        INTEGER NOT NULL REFERENCES item_type(type_id) ON DELETE CASCADE,
    total_count    BIGINT  NOT NULL DEFAULT 0,
    archived_count BIGINT  NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, type_id)
);

INSERT INTO item_type_count (account_id, type_id, total_count, archived_count)
SELECT i.account_id, l.type_id, COUNT(*), COUNT(a.item_id)
FROM item_item_type_link l
JOIN item i ON i.item_id = l.item_id
LEFT JOIN attribute a ON a.item_id = i.item_id AND a.key = 'Archived' AND a.value_int = 1
GROUP BY i.account_id, l.type_id;

-- The link triggers run once per statement over the statement's transition table,
-- so a bulk import or restore adds up its rows first and updates each counter
-- once. One update per row would rewrite the same few counter rows a million
-- times inside one transaction, which is quadratic in PostgreSQL.
CREATE FUNCTION item_type_count_link_insert() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO item_type_count (account_id, type_id, total_count, archived_count)
    SELECT i.account_id, n.type_id, COUNT(*), COUNT(a.item_id)
    FROM new_links n
    JOIN item i ON i.item_id = n.item_id
    LEFT JOIN attribute a ON a.item_id = i.item_id AND a.key = 'Archived' AND a.value_int = 1
    GROUP BY i.account_id, n.type_id
    ON CONFLICT (account_id, type_id) DO UPDATE SET
        total_count = item_type_count.total_count + EXCLUDED.total_count,
        archived_count = item_type_count.archived_count + EXCLUDED.archived_count;
    RETURN NULL;
END $$;

-- Rows of an item that is itself being deleted find no item here: its links were
-- already taken out of the counts by item_type_count_item_delete. Rows of a type
-- being deleted find no type: the type's count rows go with it, and updating them
-- in the same transaction would re-check their foreign key.
CREATE FUNCTION item_type_count_link_delete() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    UPDATE item_type_count c SET
        total_count = c.total_count - d.total,
        archived_count = c.archived_count - d.archived
    FROM (
        SELECT i.account_id, o.type_id, COUNT(*) AS total, COUNT(a.item_id) AS archived
        FROM old_links o
        JOIN item i ON i.item_id = o.item_id
        JOIN item_type t ON t.type_id = o.type_id
        LEFT JOIN attribute a ON a.item_id = i.item_id AND a.key = 'Archived' AND a.value_int = 1
        GROUP BY i.account_id, o.type_id
    ) d
    WHERE c.account_id = d.account_id AND c.type_id = d.type_id;
    RETURN NULL;
END $$;

-- Skips an account being deleted for the same reason: its count rows are cascading away.
CREATE FUNCTION item_type_count_item_delete() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    UPDATE item_type_count SET
        total_count = total_count - 1,
        archived_count = archived_count - (EXISTS (
            SELECT 1 FROM attribute a
            WHERE a.item_id = OLD.item_id AND a.key = 'Archived' AND a.value_int = 1))::int
    WHERE account_id = OLD.account_id
      AND type_id IN (SELECT type_id FROM item_item_type_link WHERE item_id = OLD.item_id)
      AND EXISTS (SELECT 1 FROM account WHERE account_id = OLD.account_id);
    RETURN OLD;
END $$;

-- Shifts archived_count by `delta` for every type of the item.
CREATE FUNCTION item_type_count_shift_archived(target_item integer, delta integer) RETURNS void
LANGUAGE sql AS $$
    UPDATE item_type_count SET archived_count = archived_count + delta
    WHERE account_id = (SELECT account_id FROM item WHERE item_id = target_item)
      AND type_id IN (SELECT type_id FROM item_item_type_link WHERE item_id = target_item);
$$;

CREATE FUNCTION item_type_count_archive_insert() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    PERFORM item_type_count_shift_archived(NEW.item_id, 1);
    RETURN NULL;
END $$;

CREATE FUNCTION item_type_count_archive_update() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    PERFORM item_type_count_shift_archived(
        NEW.item_id,
        (NEW.key = 'Archived' AND COALESCE(NEW.value_int, 0) = 1)::int
        - (OLD.key = 'Archived' AND COALESCE(OLD.value_int, 0) = 1)::int);
    RETURN NULL;
END $$;

CREATE FUNCTION item_type_count_archive_delete() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    PERFORM item_type_count_shift_archived(OLD.item_id, -1);
    RETURN NULL;
END $$;

CREATE TRIGGER trg_type_count_link_insert AFTER INSERT ON item_item_type_link
    REFERENCING NEW TABLE AS new_links
    FOR EACH STATEMENT EXECUTE FUNCTION item_type_count_link_insert();
CREATE TRIGGER trg_type_count_link_delete AFTER DELETE ON item_item_type_link
    REFERENCING OLD TABLE AS old_links
    FOR EACH STATEMENT EXECUTE FUNCTION item_type_count_link_delete();
CREATE TRIGGER trg_type_count_item_delete BEFORE DELETE ON item
    FOR EACH ROW EXECUTE FUNCTION item_type_count_item_delete();
CREATE TRIGGER trg_type_count_archive_insert AFTER INSERT ON attribute
    FOR EACH ROW WHEN (NEW.key = 'Archived' AND NEW.value_int = 1)
    EXECUTE FUNCTION item_type_count_archive_insert();
CREATE TRIGGER trg_type_count_archive_update AFTER UPDATE ON attribute
    FOR EACH ROW WHEN ((OLD.key = 'Archived' AND COALESCE(OLD.value_int, 0) = 1)
        IS DISTINCT FROM (NEW.key = 'Archived' AND COALESCE(NEW.value_int, 0) = 1))
    EXECUTE FUNCTION item_type_count_archive_update();
CREATE TRIGGER trg_type_count_archive_delete AFTER DELETE ON attribute
    FOR EACH ROW WHEN (OLD.key = 'Archived' AND OLD.value_int = 1)
    EXECUTE FUNCTION item_type_count_archive_delete();
