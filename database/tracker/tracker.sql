create table tracker_entry (
	tracker_id bigserial primary key,
	item_id int not null references item(item_id),
	timestamp timestamptz not null default now(),
	level int not null default 3,
	comment text
);