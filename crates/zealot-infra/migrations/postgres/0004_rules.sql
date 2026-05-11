create table rule (
    rule_id        bigserial   primary key,
    account_id     bigint      not null references account(account_id) on delete cascade,
    name           text        not null,
    description    text        not null default '',
    trigger_kind   text        not null,
    trigger_config text        not null default '{}',
    script         text        not null default '',
    enabled        boolean     not null default true,
    created_at     timestamptz not null,
    last_run_at    timestamptz,
    last_error     text,
    last_output    text
);
create index idx_rule_account     on rule(account_id);
create index idx_rule_event_rules on rule(enabled, trigger_kind);
