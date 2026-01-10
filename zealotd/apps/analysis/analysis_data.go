package analysis

import (
	"time"
	"zealotd/web"
)

type ScoreEntry struct {
	Score int `json:"score"`
	Timestamp time.Time `json:"timestamp"`
}

func LastThirtyDays(accountID int) ([]ScoreEntry, error) {
	query := `
	with date_attr as (
		select item_id, value_date as timestamp
		from attribute
		where key = 'Date'
		  and value_date is not null
	),
	score_attr as (
		select item_id, value_int as score
		from attribute
		where key = 'Score'
		  and value_int is not null
	)
	select date_trunc('day', d.timestamp) as timestamp,
	       s.score
	from item i
	inner join date_attr d on d.item_id = i.item_id
	inner join score_attr s on s.item_id = i.item_id
	where i.account_id = $1
	  and d.timestamp >= now() - interval '30 days'
	group by 1, 2
	order by 1, 2;
	`
	rows, err := web.Database.Query(query, accountID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	entries := make([]ScoreEntry, 0)
	for rows.Next() {
		var entry ScoreEntry
		if err := rows.Scan(&entry.Timestamp, &entry.Score); err != nil {
			return nil, err
		}
		entries = append(entries, entry)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}

	return entries, nil
}
