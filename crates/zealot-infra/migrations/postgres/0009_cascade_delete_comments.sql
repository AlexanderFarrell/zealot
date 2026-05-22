ALTER TABLE comment
    DROP CONSTRAINT comment_item_id_fkey;

ALTER TABLE comment
    ADD CONSTRAINT comment_item_id_fkey
    FOREIGN KEY (item_id) REFERENCES item(item_id) ON DELETE CASCADE;
