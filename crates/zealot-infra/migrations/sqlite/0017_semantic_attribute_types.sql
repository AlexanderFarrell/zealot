-- Allow the email, telephone, url, media and place attribute kind types (stored as text).
-- SQLite cannot alter a CHECK constraint, so rebuild the table. Dropping it cascades to
-- item_type_attribute_kind_link (foreign keys are on), so save and restore those rows.
create temp table link_backup as
select attribute_kind_id, item_type_id from item_type_attribute_kind_link;

create table attribute_kind_new (
    kind_id integer primary key,
    account_id integer references account(account_id) on delete cascade,
    key text not null,
    description text not null default '',
    base_type text not null check (
        base_type in
        ('integer', 'decimal', 'text', 'date', 'week', 'dropdown', 'boolean', 'list', 'item',
         'email', 'telephone', 'url', 'media', 'place')
    ),
    config text not null default '{}',
    is_system integer not null default 0,
    constraint u_account_key unique (account_id, key),
    constraint c_account_or_system check (
        (account_id is not null and not is_system) or (is_system and account_id is null)
    )
);

insert into attribute_kind_new (kind_id, account_id, key, description, base_type, config, is_system)
select kind_id, account_id, key, description, base_type, config, is_system from attribute_kind;

drop table attribute_kind;
alter table attribute_kind_new rename to attribute_kind;

insert into item_type_attribute_kind_link (attribute_kind_id, item_type_id)
select attribute_kind_id, item_type_id from link_backup;

drop table link_backup;
