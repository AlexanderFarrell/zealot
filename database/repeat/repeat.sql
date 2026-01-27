create type repeat_entry_status as enum (
	'Complete',
	'Skip',
	'Alternate'
);


create table repeat_entry (
	repeat_id bigserial primary key,
	item_id int not null references item(item_id)
		on delete cascade,
	date date not null,
	status repeat_entry_status not null default 'Complete',
	comment text
);

