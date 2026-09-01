insert into attribute_kind (key, description, base_type, is_system, config)
select 'Value Kind', 'How statistic values are presented.', 'dropdown', 1,
       '{"values": ["Number", "Duration", "Ordinal"]}'
where not exists (select 1 from attribute_kind where key = 'Value Kind' and is_system = 1);

insert into attribute_kind (key, description, base_type, is_system, config)
select 'Unit', 'Unit displayed alongside statistic values.', 'text', 1, '{}'
where not exists (select 1 from attribute_kind where key = 'Unit' and is_system = 1);

insert into attribute_kind (key, description, base_type, is_system, config)
select 'Daily Aggregation', 'How entries are combined for one UTC day.', 'dropdown', 1,
       '{"values": ["Sum", "Average", "Minimum", "Maximum", "Latest", "None"]}'
where not exists (select 1 from attribute_kind where key = 'Daily Aggregation' and is_system = 1);

insert into item_type (name, description, account_id)
select 'Statistic', 'A typed numeric series.', null
where not exists (select 1 from item_type where name = 'Statistic' and account_id is null);

insert or ignore into item_type_attribute_kind_link (attribute_kind_id, item_type_id)
select ak.kind_id, it.type_id
from attribute_kind ak
cross join item_type it
where ak.key in ('Value Kind', 'Unit', 'Daily Aggregation')
  and ak.is_system = 1
  and it.name = 'Statistic'
  and it.account_id is null;

create table statistic_entry (
    statistic_entry_id integer primary key,
    item_id integer not null references item(item_id) on delete cascade,
    account_id integer not null references account(account_id) on delete cascade,
    value real not null check (abs(value) <= 1.7976931348623157e308),
    occurred_at integer not null,
    related_item_id integer references item(item_id) on delete set null,
    comment text,
    created_at integer not null default (strftime('%s', 'now')),
    updated_at integer not null default (strftime('%s', 'now'))
);

create index idx_statistic_entry_item_time
    on statistic_entry (account_id, item_id, occurred_at desc, statistic_entry_id desc);
