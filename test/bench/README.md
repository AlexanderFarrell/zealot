# Performance Benchmark

Measures how long edits and queries take on a database of up to one million
items, on SQLite and PostgreSQL, using a release build of the server. It backs
the Z-E7 requirement "edits within 500 ms on a database of 1M items".

## Run it

```sh
./test/bench/run.sh sqlite                       # 1k, 10k, 100k, 1M items
./test/bench/run.sh postgres --scales 1000,10000 # a smaller ladder
./test/bench/run.sh sqlite --scales 100000 --samples 300 --clients 8
```

Needs `cargo`, `python3`, `curl`, and `sqlite3` (SQLite runs) or `docker`
(PostgreSQL runs). The first run creates `test/bench/.venv` and builds
`target/release/zealot-server` if it is missing; set `ZEALOT_SERVER_BIN` to use
another binary.

Everything after the scales is passed to `bench.py run`:

| option | default | meaning |
| --- | --- | --- |
| `--samples` | 200 | requests per operation (single client) |
| `--heavy-samples` | 10 | requests for the unpaginated type screen list |
| `--warmup` | 20 | untimed requests before each operation |
| `--clients` | 4 | threads for the concurrent pass (edits and one filter) |
| `--only a,b` | all | run only these operations |

It never touches a real Zealot: databases live in `test/bench/.work` (SQLite) or
in a throwaway `postgres:16-alpine` container (`zealot-bench-pg`, port 47214,
`shared_buffers=1GB`, `synchronous_commit=on`), and the server listens on port
47213. Override with `BENCH_PORT`, `BENCH_PG_PORT`, `BENCH_PG_IMAGE`,
`BENCH_WORK_DIR`, `BENCH_RESULTS_DIR`.

Each scale runs from a fresh database: start the server (migrations), create the
benchmark account, attribute kinds and types (`bench.py prepare`), stop the
server, seed with SQL, start the server, verify (`bench.py check`), measure
(`bench.py run`). Results go to `test/bench/results/<db>-<items>.json` and
`.md`, with seed time and database size.

## Seed data

`seed.sqlite.sql` and `seed.postgres.sql` fill the tables directly with
set-based SQL (HTTP creation takes minutes per 10k items). They are
deterministic, so the same N gives the same data, and are modeled on the
proportions of a real Zealot:

- item `i` is titled `Item 0000042 Gamma` (unique); content is 0, 200, 1000 or
  2000 characters; every ninth item links to another item with `[[Title]]`.
- types by `i % 12`: Ticket, Topic, Book, Word, Recipe, Project, Epic, Document,
  Meeting, Repo, Game, and none. Every tenth item is also a Topic.
- typed items have Status and Priority; half have Description; Tickets have
  TicketID; Books have Rating (all required attributes of their types are set).
- Parent on 54% of items (tree with fan-out 10; about 3% have a second parent);
  Topic list on about 18% (2 to 3 each); `item_item_link` rows for parent, topic
  and wikilink are inserted to match.

`bench.py check` compares the seeded database with what the services would
produce (counts, titles, Parent attribute against parent link rows, wikilink
rows) before any measurement. If you change a seed script, keep `title_of`,
`typed_id` and `untyped_id` in `bench.py` in step.

## Operations

Random existing items, one client, sequential, then a second pass with
`--clients` threads for the edits and one filter. Every response must be 2xx or
it counts as an error.

`get_item`, `children`, `filter_type_attr` (`POST /item/query`, type plus
Status), `filter_attr_sorted`, `query_under` (subtree), `type_summary`,
`type_screen_list` (`GET /item?type=`, what the type screen loads: the whole
list), `create_item`, `attr_set_scalar`, `attr_set_parent`, `type_assign`,
`content_edit` and `content_edit_wikilink` (`PATCH /item/{id}`), and
`title_edit` last.

**Abort rule.** An operation stops after 3 consecutive samples above 30 s, or
after 600 s in total, and is reported as exceeding its budget with the samples
completed. Requests time out at 60 s and count as errors. A pathological
operation is therefore a result, not a hang. Title edit runs last because it can
leave the server busy for a long time.

## Reading results

Report hardware, commit, build, database and Docker caveats with the numbers.
PostgreSQL runs in Docker Desktop's Linux VM on macOS, so its figures include VM
and port-forward latency. One machine and one client measure whether the target
is plausible on this hardware, not production capacity.
