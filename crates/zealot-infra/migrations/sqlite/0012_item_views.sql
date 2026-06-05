CREATE TABLE item_view (
    view_id   INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id   INTEGER NOT NULL REFERENCES item(item_id) ON DELETE CASCADE,
    viewed_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX idx_item_view_item_id ON item_view(item_id);
