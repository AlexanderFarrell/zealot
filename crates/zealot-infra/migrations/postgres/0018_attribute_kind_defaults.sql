-- System kinds are read-only through the API, so their defaults ship here. Kinds store `default` in config.
UPDATE attribute_kind SET config = jsonb_set(config, '{default}', '"To Do"') WHERE key = 'Status' AND is_system;
UPDATE attribute_kind SET config = jsonb_set(config, '{default}', '3') WHERE key = 'Priority' AND is_system;
