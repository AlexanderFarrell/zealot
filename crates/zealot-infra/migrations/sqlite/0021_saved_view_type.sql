-- Saved View: an item whose content holds a fenced `zealot-view` query definition (Z161).
insert into item_type (name, description, account_id)
select 'Saved View', 'A saved, re-runnable item query defined in a zealot-view block.', null
where not exists (select 1 from item_type where name = 'Saved View' and account_id is null);
