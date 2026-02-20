

create table comment(
	comment_id bigserial primary key,
	item_id int not null references item(item_id),
	time timestamptz not null default now(),
	content text not null default '',
	created_on timestamptz not null default now(),
	last_updated timestamptz not null default now()
);

