-- SQLite does not support ALTER TABLE DROP CONSTRAINT, so we recreate
-- the comment table with ON DELETE CASCADE on the item_id foreign key.

PRAGMA foreign_keys = OFF;

CREATE TABLE comment_new (
    comment_id integer primary key,
    item_id integer not null references item(item_id) on delete cascade,
    time integer not null default (strftime('%s', 'now')),
    content text not null default '',
    created_on integer not null default (strftime('%s', 'now')),
    last_updated integer not null default (strftime('%s', 'now'))
);

INSERT INTO comment_new (comment_id, item_id, time, content, created_on, last_updated)
SELECT comment_id, item_id, time, content, created_on, last_updated
FROM comment;

DROP TABLE comment;
ALTER TABLE comment_new RENAME TO comment;

PRAGMA foreign_keys = ON;
