PRAGMA foreign_keys = ON;

-- 

create table if not exists item (
    item_id integer primary key,
    title text not null,
    kind text,
    icon text,
    content text default '' not null,
    status text,
    date int,
    created_on int not null default (strftime('%s', 'now')),
    updated_at int not null default (strftime('%s', 'now')),
    week int,
    year int,
    parent_id int references item(item_id)
);

create table if not exists metadata (
    meta_id integer primary key,
    key text not null,
    value text default '' not null,
    item_id integer references item(item_id) not null
);