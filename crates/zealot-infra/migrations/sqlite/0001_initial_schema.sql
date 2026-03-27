-- SQLite initial schema.
-- Note: PRAGMA foreign_keys is set via SqliteConnectOptions at pool creation, not here,
-- because SQLite pragmas cannot run inside a transaction (sqlx wraps each migration in one).

create table account (
    account_id integer primary key,
    username text not null,
    email text not null,
    password text not null,
    given_name text not null default '',
    surname text not null default '',
    settings text not null default '{}',
    created_on integer not null default (strftime('%s', 'now'))
);

create table item (
    item_id integer primary key,
    title text not null default '',
    content text not null default '',
    account_id integer not null references account(account_id) on delete cascade,
    created_at integer not null default (strftime('%s', 'now')),
    updated_at integer not null default (strftime('%s', 'now'))
);

create table attribute (
    item_id integer not null references item(item_id) on delete cascade,
    key text not null,
    value_text text,
    value_num real,
    value_int integer,
    value_date integer,
    value_item_id integer references item(item_id) on delete cascade,
    primary key (item_id, key)
);

create table attribute_list_value (
    alv_id integer primary key,
    item_id integer not null references item(item_id) on delete cascade,
    key text not null,
    ordinal integer not null default 0,
    value_text text,
    value_num real,
    value_int integer,
    value_date integer,
    value_item_id integer references item(item_id) on delete cascade
);

create table attribute_kind (
    kind_id integer primary key,
    account_id integer references account(account_id) on delete cascade,
    key text not null,
    description text not null default '',
    base_type text not null check (
        base_type in
        ('integer', 'decimal', 'text', 'date', 'week', 'dropdown', 'boolean', 'list', 'item')
    ),
    config text not null default '{}',
    is_system integer not null default 0,
    constraint u_account_key unique (account_id, key),
    constraint c_account_or_system check (
        (account_id is not null and not is_system) or (is_system and account_id is null)
    )
);

create index idx_attribute_key_text on attribute (key, value_text);
create index idx_attribute_key_num on attribute (key, value_num);
create index idx_attribute_key_int on attribute (key, value_int);
create index idx_attribute_key_date on attribute (key, value_date);
create index idx_attribute_key_item on attribute (key, value_item_id);

create index idx_alv_key_text on attribute_list_value (key, value_text);
create index idx_alv_key_int on attribute_list_value (key, value_int);
create index idx_alv_key_num on attribute_list_value (key, value_num);
create index idx_alv_key_date on attribute_list_value (key, value_date);
create index idx_alv_key_item on attribute_list_value (key, value_item_id);
create index idx_alv_item_key on attribute_list_value (item_id, key);

create table item_type (
    type_id integer primary key,
    name text not null,
    description text not null,
    account_id integer references account(account_id) on delete cascade,
    constraint u_name_account unique (name, account_id)
);

insert into item_type (name, description, account_id)
values
('Plan', 'Something to get done', null),
('Repeat', 'A task which repeats', null);

create table item_item_type_link (
    item_id integer not null references item(item_id) on delete cascade,
    type_id integer not null references item_type(type_id) on delete cascade,
    primary key (item_id, type_id)
);

create table item_type_attribute_kind_link (
    attribute_kind_id integer not null references attribute_kind(kind_id) on delete cascade,
    item_type_id integer not null references item_type(type_id) on delete cascade,
    primary key (attribute_kind_id, item_type_id)
);

-- item_relationship: SQLite has no enum type; use a CHECK constraint instead
create table item_item_link (
    item_item_id integer primary key,
    first_item_id integer not null references item(item_id) on delete cascade,
    second_item_id integer not null references item(item_id) on delete cascade,
    relationship text not null default 'Parent'
        check (relationship in ('Parent', 'Blocks', 'Tag', 'Topic', 'Other')),
    constraint u_item_link unique (first_item_id, second_item_id)
);

-- repeat_entry_status: inline CHECK instead of enum
create table repeat_entry (
    repeat_id integer primary key,
    item_id integer not null references item(item_id) on delete cascade,
    date text not null,
    status text not null default 'Complete'
        check (status in ('Complete', 'Skip', 'Alternate')),
    comment text
);

create table comment (
    comment_id integer primary key,
    item_id integer not null references item(item_id),
    time integer not null default (strftime('%s', 'now')),
    content text not null default '',
    created_on integer not null default (strftime('%s', 'now')),
    last_updated integer not null default (strftime('%s', 'now'))
);

create table if not exists session (
    token_hash  text    primary key,
    account_id  integer not null references account(account_id) on delete cascade,
    expires_at  integer not null
);

create index if not exists idx_session_account_id on session(account_id);

-- Seed data: system attribute kinds
insert into attribute_kind (key, description, base_type, is_system, config)
values
('Date', 'When to do something.', 'date', 1, '{}'),
('Status', 'Where is the current item?', 'dropdown', 1,
    '{"values": ["To Do", "Specify", "Working", "Hold", "Complete", "Rejected", "Blocked"]}'),
('Week', 'What week to do something?', 'week', 1, '{}'),
('Priority', 'How important is something?', 'integer', 1, '{"min": 1, "max": 10}'),
('Month', 'Tentative month something is scheduled on', 'integer', 1, '{}'),
('Year', 'Tentative year something is scheduled on', 'integer', 1, '{}'),
('Root', 'Does the item appear at the top of the nav view?', 'boolean', 1, '{}'),
('Parent', 'Creates hierarchical relationships', 'list', 1, '{"list_type": "text"}');

insert into attribute_kind (key, description, base_type, is_system, config)
select 'Time of Day', 'The general time of day which something occurred.', 'dropdown', 1,
    '{"values": ["Morning", "Afternoon", "Evening", "Anytime"]}'
where not exists (select 1 from attribute_kind where key = 'Time of Day' and is_system = 1);

insert into attribute_kind (key, description, base_type, is_system, config)
select 'Phone', 'International phone number (E.164).', 'text', 1,
    '{"pattern": "[+][1-9][0-9]{1,14}"}'
where not exists (select 1 from attribute_kind where key = 'Phone' and is_system = 1);

insert into attribute_kind (key, description, base_type, is_system, config)
select 'Email', 'Email address.', 'text', 1,
    '{"pattern": "[^@]+@[^@]+[.][^@]+"}'
where not exists (select 1 from attribute_kind where key = 'Email' and is_system = 1);

insert into attribute_kind (key, description, base_type, is_system, config)
select 'Schedule', 'Configuration for how often something should happen', 'text', 1,
    '{"pattern": "[0-1]{10}"}'
where not exists (select 1 from attribute_kind where key = 'Schedule' and is_system = 1);

insert into attribute_kind (key, description, base_type, is_system, config)
select 'End Date', 'The date at which something stops.', 'date', 1, '{}'
where not exists (select 1 from attribute_kind where key = 'End Date' and is_system = 1);

-- Seed data: attribute kind links for default item types
insert into item_type_attribute_kind_link (attribute_kind_id, item_type_id)
values
(
    (select kind_id from attribute_kind where key = 'Status'),
    (select type_id from item_type where name = 'Plan')
),
(
    (select kind_id from attribute_kind where key = 'Priority'),
    (select type_id from item_type where name = 'Plan')
),
(
    (select kind_id from attribute_kind where key = 'Schedule'),
    (select type_id from item_type where name = 'Repeat')
);
