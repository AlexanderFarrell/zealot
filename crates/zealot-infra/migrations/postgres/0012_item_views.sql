CREATE TABLE item_view (
    view_id   SERIAL PRIMARY KEY,
    item_id   INTEGER NOT NULL REFERENCES item(item_id) ON DELETE CASCADE,
    viewed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_item_view_item_id ON item_view(item_id);
