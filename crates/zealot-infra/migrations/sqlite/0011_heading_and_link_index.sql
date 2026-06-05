CREATE TABLE item_heading (
    heading_id  INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id     INTEGER NOT NULL REFERENCES item(item_id) ON DELETE CASCADE,
    level       INTEGER NOT NULL,
    ordinal     INTEGER NOT NULL,
    text        TEXT    NOT NULL,
    CONSTRAINT u_item_heading UNIQUE (item_id, ordinal)
);

CREATE TABLE item_external_link (
    link_id  INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id  INTEGER NOT NULL REFERENCES item(item_id) ON DELETE CASCADE,
    url      TEXT    NOT NULL,
    CONSTRAINT u_item_external_link UNIQUE (item_id, url)
);
