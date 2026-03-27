create table if not exists session (
    token_hash  text    primary key,
    account_id  integer not null references account(account_id) on delete cascade,
    expires_at  integer not null  -- Unix timestamp (seconds since epoch)
);

create index if not exists idx_session_account_id on session(account_id);
