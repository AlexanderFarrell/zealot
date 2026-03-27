-- Split account.full_name into given_name + surname.
-- Existing rows are migrated via a naive first-space split.

alter table account add column given_name varchar(100) not null default '';
alter table account add column surname varchar(100) not null default '';

update account set
    given_name = case
        when full_name like '% %' then split_part(full_name, ' ', 1)
        else full_name
    end,
    surname = case
        when full_name like '% %'
            then substring(full_name from position(' ' in full_name) + 1)
        else ''
    end;

alter table account drop column full_name;
