begin;

-- Migrate tracker entries into comments.
-- Maps tracker_entry.timestamp -> comment.time
-- Maps tracker_entry.comment   -> comment.content
insert into comment (item_id, time, content, created_on, last_updated)
select
	te.item_id,
	te.timestamp,
	coalesce(te.comment, ''),
	now(),
	now()
from tracker_entry te;

commit;
