create table rule (
    rule_id        integer primary key,
    account_id     integer not null references account(account_id) on delete cascade,
    name           text    not null,
    description    text    not null default '',
    trigger_kind   text    not null,
    trigger_config text    not null default '{}',
    script         text    not null default '',
    enabled        integer not null default 1,
    created_at     integer not null,
    last_run_at    integer,
    last_error     text,
    last_output    text
);
create index idx_rule_account     on rule(account_id);
create index idx_rule_event_rules on rule(enabled, trigger_kind);
