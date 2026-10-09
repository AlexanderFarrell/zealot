-- Z223: indexes for 1M-item databases, and the wikilink reference table that
-- lets a rename re-resolve only the links it affects.

CREATE INDEX idx_item_account_title ON item (account_id, title);
CREATE INDEX idx_item_link_second_rel ON item_item_link (second_item_id, relationship);
CREATE INDEX idx_item_type_link_type ON item_item_type_link (type_id, item_id);

-- Every [[Title]] an item's content mentions, resolved or not.
CREATE TABLE item_wikilink_ref (
    item_id      INTEGER NOT NULL REFERENCES item(item_id) ON DELETE CASCADE,
    target_title TEXT    NOT NULL,
    PRIMARY KEY (item_id, target_title)
);
CREATE INDEX idx_wikilink_ref_title ON item_wikilink_ref (target_title);

-- An account is listed once item_wikilink_ref covers all of its items
-- (written by a full wikilink rebuild). Until then a rename rebuilds in full.
CREATE TABLE wikilink_ref_state (
    account_id INTEGER PRIMARY KEY REFERENCES account(account_id) ON DELETE CASCADE
);
