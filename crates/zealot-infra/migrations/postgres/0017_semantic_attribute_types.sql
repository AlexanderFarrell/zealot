-- Allow the email, telephone, url, media and place attribute kind types (stored as text).
alter table attribute_kind drop constraint attribute_kind_base_type_check;
alter table attribute_kind add constraint attribute_kind_base_type_check check (
    base_type in
    ('integer', 'decimal', 'text', 'date', 'week', 'dropdown', 'boolean', 'list', 'item',
     'email', 'telephone', 'url', 'media', 'place')
);
