-- SQLite does not support ALTER COLUMN or DROP CONSTRAINT, so we recreate
-- item_item_link without the CHECK constraint and with the relationship stored
-- in lowercase, then update the unique key to include relationship.

PRAGMA foreign_keys = OFF;

CREATE TABLE item_item_link_new (
    item_item_id integer primary key,
    first_item_id integer not null references item(item_id) on delete cascade,
    second_item_id integer not null references item(item_id) on delete cascade,
    relationship text not null default 'parent',
    constraint u_item_link unique (first_item_id, second_item_id, relationship)
);

INSERT INTO item_item_link_new (item_item_id, first_item_id, second_item_id, relationship)
SELECT item_item_id, first_item_id, second_item_id, lower(relationship)
FROM item_item_link;

DROP TABLE item_item_link;
ALTER TABLE item_item_link_new RENAME TO item_item_link;

PRAGMA foreign_keys = ON;

-- Upgrade the Parent attribute kind from list-of-text to list-of-item.
UPDATE attribute_kind
SET    config = '{"list_type": "item"}'
WHERE  key = 'Parent';
