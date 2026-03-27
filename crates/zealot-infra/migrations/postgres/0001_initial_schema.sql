-- Extensions (enable manually if needed):
-- create extension if not exists pgcrypto;

create table account (
    account_id serial primary key,
    username varchar(30) not null,
    email varchar(128) not null,
    password varchar(128) not null,
    full_name varchar(100) not null,
    settings jsonb not null default '{}',
    created_on timestamptz not null default now()
);

comment on table account is
    'Stores users, hashed passwords, and user info';
comment on column account.account_id is
    'Unique ID for the account, used for linking to the account';
comment on column account.username is
    'The name of the account when logging in';
comment on column account.email is
    'The email of the user of the account';
comment on column account.password is
    'The HASHED password of the user account for authentication';
comment on column account.full_name is
    'The full name of the user, first name and last name';

create table item (
    item_id serial primary key,
    title varchar(200) not null default '',
    content text not null default '',
    account_id int not null references account(account_id) on delete cascade,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table attribute (
    item_id integer not null references item(item_id) on delete cascade,
    key text not null,
    value_text text,
    value_num real,
    value_int integer,
    value_date timestamptz,
    value_item_id integer references item(item_id) on delete cascade,
    primary key (item_id, key)
);

create table attribute_list_value (
    alv_id serial primary key,
    item_id integer not null references item(item_id) on delete cascade,
    key text not null,
    ordinal int not null default 0,
    value_text text,
    value_num real,
    value_int int,
    value_date timestamptz,
    value_item_id integer references item(item_id) on delete cascade
);

create table attribute_kind (
    kind_id serial primary key,
    account_id int references account(account_id) on delete cascade,
    key text not null,
    description text not null default '',
    base_type text not null check (
        base_type in
        ('integer', 'decimal', 'text', 'date', 'week', 'dropdown', 'boolean', 'list', 'item')
    ),
    config jsonb not null default '{}'::jsonb,
    is_system boolean not null default false,
    constraint u_account_key unique (account_id, key),
    constraint c_account_or_system check (
        (account_id is not null and not is_system) or (is_system and account_id is null)
    )
);

create index on attribute (key, value_text);
create index on attribute (key, value_num);
create index on attribute (key, value_int);
create index on attribute (key, value_date);
create index on attribute (key, value_item_id);

create index on attribute_list_value (key, value_text);
create index on attribute_list_value (key, value_int);
create index on attribute_list_value (key, value_num);
create index on attribute_list_value (key, value_date);
create index on attribute_list_value (key, value_item_id);
create index on attribute_list_value (item_id, key);

create table item_type (
    type_id serial primary key,
    name text not null,
    description text not null,
    account_id int references account(account_id) on delete cascade,
    constraint u_name_account unique (name, account_id)
);

insert into item_type (name, description, account_id)
values
('Plan', 'Something to get done', null),
('Repeat', 'A task which repeats', null);

create table item_item_type_link (
    item_id int not null references item(item_id) on delete cascade,
    type_id int not null references item_type(type_id) on delete cascade,
    primary key (item_id, type_id)
);

create table item_type_attribute_kind_link (
    attribute_kind_id int not null references attribute_kind(kind_id) on delete cascade,
    item_type_id int not null references item_type(type_id) on delete cascade,
    primary key (attribute_kind_id, item_type_id)
);

create type item_relationship as enum (
    'Parent',
    'Blocks',
    'Tag',
    'Topic',
    'Other'
);

create table item_item_link (
    item_item_id serial primary key,
    first_item_id int not null references item(item_id) on delete cascade,
    second_item_id int not null references item(item_id) on delete cascade,
    relationship item_relationship not null default 'Parent',
    constraint u_item_link unique (first_item_id, second_item_id)
);

create type repeat_entry_status as enum (
    'Complete',
    'Skip',
    'Alternate'
);

create table repeat_entry (
    repeat_id bigserial primary key,
    item_id int not null references item(item_id) on delete cascade,
    date date not null,
    status repeat_entry_status not null default 'Complete',
    comment text
);

create table comment (
    comment_id bigserial primary key,
    item_id int not null references item(item_id),
    time timestamptz not null default now(),
    content text not null default '',
    created_on timestamptz not null default now(),
    last_updated timestamptz not null default now()
);

create table if not exists session (
    token_hash  text        primary key,
    account_id  bigint      not null references account(account_id) on delete cascade,
    expires_at  timestamptz not null
);

comment on table session is
    'Stores hashed session tokens. Raw tokens live only in client cookies.';
comment on column session.token_hash is
    'SHA-256 hex digest of the raw session token stored in the cookie';
comment on column session.expires_at is
    'Absolute expiry; server rejects sessions past this time';

create index if not exists idx_session_account_id on session(account_id);

-- Seed data: system attribute kinds
insert into attribute_kind (key, description, base_type, is_system, config)
values
('Date', 'When to do something.', 'date', true, '{}'),
('Status', 'Where is the current item?', 'dropdown', true,
    '{"values": ["To Do", "Specify", "Working", "Hold", "Complete", "Rejected", "Blocked"]}'),
('Week', 'What week to do something?', 'week', true, '{}'),
('Priority', 'How important is something?', 'integer', true, '{"min": 1, "max": 10}'),
('Month', 'Tentative month something is scheduled on', 'integer', true, '{}'),
('Year', 'Tentative year something is scheduled on', 'integer', true, '{}'),
('Root', 'Does the item appear at the top of the nav view?', 'boolean', true, '{}'),
('Parent', 'Creates hierarchical relationships', 'list', true, '{"list_type": "text"}');

insert into attribute_kind (key, description, base_type, is_system, config)
select 'Time of Day', 'The general time of day which something occurred.', 'dropdown', true,
    '{"values": ["Morning", "Afternoon", "Evening", "Anytime"]}'
where not exists (select 1 from attribute_kind where key = 'Time of Day' and is_system);

insert into attribute_kind (key, description, base_type, is_system, config)
select 'Phone', 'International phone number (E.164).', 'text', true,
    '{"pattern": "[+][1-9][0-9]{1,14}"}'
where not exists (select 1 from attribute_kind where key = 'Phone' and is_system);

insert into attribute_kind (key, description, base_type, is_system, config)
select 'Email', 'Email address.', 'text', true,
    '{"pattern": "[^@[:space:]]+@[^@[:space:]]+[.][^@[:space:]]+"}'
where not exists (select 1 from attribute_kind where key = 'Email' and is_system);

insert into attribute_kind (key, description, base_type, is_system, config)
select 'Schedule', 'Configuration for how often something should happen', 'text', true,
    '{"pattern": "[0-1]{10}"}'
where not exists (select 1 from attribute_kind where key = 'Schedule' and is_system);

insert into attribute_kind (key, description, base_type, is_system, config)
select 'End Date', 'The date at which something stops.', 'date', true, '{}'
where not exists (select 1 from attribute_kind where key = 'End Date' and is_system);

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
