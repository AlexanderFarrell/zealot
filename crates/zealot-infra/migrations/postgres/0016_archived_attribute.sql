-- Archive state uses the existing attribute primitive; no item data is rewritten.
INSERT INTO attribute_kind (key, description, base_type, is_system, config)
VALUES ('Archived', 'Hidden from default lists; reversible without changing item history.', 'boolean', true, '{}');
