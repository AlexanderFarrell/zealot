-- Convert the relationship column from the fixed enum to a free-form VARCHAR so
-- that user-defined attribute kinds can act as relationship types without code
-- changes.

-- 1. Drop the column default (it is typed as item_relationship, which would
--    block the type drop even after the column type is changed).
ALTER TABLE item_item_link
    ALTER COLUMN relationship DROP DEFAULT;

-- 2. Convert the enum column to VARCHAR, lowercased for consistency with the
--    JSON API which uses snake_case for relationship values.
ALTER TABLE item_item_link
    ALTER COLUMN relationship TYPE VARCHAR(64)
    USING lower(relationship::text);

-- 3. Restore a sensible default with the new type.
ALTER TABLE item_item_link
    ALTER COLUMN relationship SET DEFAULT 'parent';

-- 4. Drop the now-unused enum type (no dependencies remain).
DROP TYPE IF EXISTS item_relationship;

-- 3. Widen the unique constraint to include the relationship column so that
--    multiple relationship types can exist between the same pair of items.
ALTER TABLE item_item_link DROP CONSTRAINT u_item_link;
ALTER TABLE item_item_link ADD CONSTRAINT u_item_link
    UNIQUE (first_item_id, second_item_id, relationship);

-- 4. Upgrade the Parent attribute kind from list-of-text (legacy) to
--    list-of-item so the attribute editor renders an item-chips picker.
UPDATE attribute_kind
SET    config = '{"list_type": "item"}'
WHERE  key = 'Parent';
