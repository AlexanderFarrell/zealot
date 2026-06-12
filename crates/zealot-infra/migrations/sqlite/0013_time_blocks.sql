create table time_block (
    block_id   integer primary key,
    item_id    integer not null references item(item_id) on delete cascade,
    date       text    not null,
    start_min  integer not null,
    end_min    integer not null,
    note       text    not null default '',
    account_id integer not null references account(account_id) on delete cascade
);

create index idx_time_block_date_account on time_block (account_id, date);
create index idx_time_block_item on time_block (item_id);
