-- Z224: stored per-type item counts, so the type summary does not count about
-- a million type links on every request. Triggers keep the table correct on
-- every write path (assign, unassign, delete, archive, import, restore).

CREATE TABLE item_type_count (
    account_id     INTEGER NOT NULL REFERENCES account(account_id) ON DELETE CASCADE,
    type_id        INTEGER NOT NULL REFERENCES item_type(type_id) ON DELETE CASCADE,
    total_count    INTEGER NOT NULL DEFAULT 0,
    archived_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, type_id)
) WITHOUT ROWID;

INSERT INTO item_type_count (account_id, type_id, total_count, archived_count)
SELECT i.account_id, l.type_id, COUNT(*), COUNT(a.item_id)
FROM item_item_type_link l
JOIN item i ON i.item_id = l.item_id
LEFT JOIN attribute a ON a.item_id = i.item_id AND a.key = 'Archived' AND a.value_int = 1
GROUP BY i.account_id, l.type_id;

CREATE TRIGGER trg_type_count_link_insert AFTER INSERT ON item_item_type_link
BEGIN
    INSERT INTO item_type_count (account_id, type_id, total_count, archived_count)
    SELECT i.account_id, NEW.type_id, 1,
           EXISTS (SELECT 1 FROM attribute a
                   WHERE a.item_id = i.item_id AND a.key = 'Archived' AND a.value_int = 1)
    FROM item i WHERE i.item_id = NEW.item_id
    ON CONFLICT (account_id, type_id) DO UPDATE SET
        total_count = total_count + 1,
        archived_count = archived_count + excluded.archived_count;
END;

-- No-op while the item itself is being deleted: trg_type_count_item_delete has
-- already taken its links out of the counts, and the item row is gone by then.
CREATE TRIGGER trg_type_count_link_delete AFTER DELETE ON item_item_type_link
BEGIN
    UPDATE item_type_count SET
        total_count = total_count - 1,
        archived_count = archived_count - EXISTS (
            SELECT 1 FROM attribute a
            WHERE a.item_id = OLD.item_id AND a.key = 'Archived' AND a.value_int = 1)
    WHERE type_id = OLD.type_id
      AND account_id = (SELECT account_id FROM item WHERE item_id = OLD.item_id);
END;

CREATE TRIGGER trg_type_count_item_delete BEFORE DELETE ON item
BEGIN
    UPDATE item_type_count SET
        total_count = total_count - 1,
        archived_count = archived_count - EXISTS (
            SELECT 1 FROM attribute a
            WHERE a.item_id = OLD.item_id AND a.key = 'Archived' AND a.value_int = 1)
    WHERE account_id = OLD.account_id
      AND type_id IN (SELECT type_id FROM item_item_type_link WHERE item_id = OLD.item_id);
END;

CREATE TRIGGER trg_type_count_archive_insert AFTER INSERT ON attribute
WHEN NEW.key = 'Archived' AND NEW.value_int = 1
BEGIN
    UPDATE item_type_count SET archived_count = archived_count + 1
    WHERE account_id = (SELECT account_id FROM item WHERE item_id = NEW.item_id)
      AND type_id IN (SELECT type_id FROM item_item_type_link WHERE item_id = NEW.item_id);
END;

CREATE TRIGGER trg_type_count_archive_update AFTER UPDATE ON attribute
WHEN (OLD.key = 'Archived' AND COALESCE(OLD.value_int, 0) = 1)
  != (NEW.key = 'Archived' AND COALESCE(NEW.value_int, 0) = 1)
BEGIN
    UPDATE item_type_count SET
        archived_count = archived_count
            + (NEW.key = 'Archived' AND COALESCE(NEW.value_int, 0) = 1)
            - (OLD.key = 'Archived' AND COALESCE(OLD.value_int, 0) = 1)
    WHERE account_id = (SELECT account_id FROM item WHERE item_id = NEW.item_id)
      AND type_id IN (SELECT type_id FROM item_item_type_link WHERE item_id = NEW.item_id);
END;

CREATE TRIGGER trg_type_count_archive_delete AFTER DELETE ON attribute
WHEN OLD.key = 'Archived' AND OLD.value_int = 1
BEGIN
    UPDATE item_type_count SET archived_count = archived_count - 1
    WHERE account_id = (SELECT account_id FROM item WHERE item_id = OLD.item_id)
      AND type_id IN (SELECT type_id FROM item_item_type_link WHERE item_id = OLD.item_id);
END;
