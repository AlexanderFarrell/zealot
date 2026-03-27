# Database Migrations

Zealot manages schema changes through versioned SQL migration files. Migrations run automatically at server startup — there is no separate migration step to run manually.

## Where migration files live

```
crates/zealot-infra/migrations/
├── postgres/
│   ├── 0001_initial_schema.sql
│   └── 0002_account_split_full_name.sql
└── sqlite/
    ├── 0001_initial_schema.sql
    └── 0002_account_split_full_name.sql
```

Each supported database engine has its own subdirectory. When you add a schema change, you generally need a file in both `postgres/` and `sqlite/` — or just the one that applies if the change is engine-specific.

## Naming convention

```
{VERSION}_{description}.sql
```

- **VERSION** — a zero-padded integer (`0001`, `0002`, …). Migrations are applied in ascending version order.
- **description** — a short snake_case summary of what the migration does.

Examples:
```
0003_add_item_tags.sql
0004_index_session_expires_at.sql
```

The description you choose here is what gets stored in the `_sqlx_migrations` tracking table, with underscores converted to spaces (e.g. `"add item tags"`).

## How Zealot applies them automatically

At startup, `zealot-infra` connects to the database and immediately calls `sqlx::migrate!` before the server accepts any traffic:

```
startup
  └── get_repo_from_config()          (zealot-infra/src/repos/mod.rs)
        ├── connect to database
        ├── sqlx::migrate!("migrations/postgres")   -- or sqlite
        │     ├── reads _sqlx_migrations table
        │     ├── skips already-applied versions
        │     └── runs new files in version order, each in a transaction
        └── return connection pool to the rest of the app
```

If any migration fails the server exits with an error — it will not start in a partially-migrated state.

## The `_sqlx_migrations` tracking table

sqlx creates and owns this table automatically. You should never edit it by hand.

| column | description |
|---|---|
| `version` | integer version number from the filename prefix |
| `description` | filename description with underscores as spaces |
| `installed_on` | timestamp when the migration was applied |
| `success` | whether the migration completed without error |
| `checksum` | hash of the file contents — changes to an already-applied file cause startup to fail |

**Do not edit applied migration files.** sqlx checksums every file; modifying one after it has run will cause the server to refuse to start. If you need to undo a change, write a new migration that reverses it.

## Writing a new migration

1. Determine the next version number by looking at the highest existing one in the directory.
2. Create the file in both `postgres/` and `sqlite/` (or whichever engines are applicable).
3. Write plain SQL. Each migration runs inside a transaction, so either the whole file succeeds or the whole file rolls back.
4. If a change is impossible or irrelevant on one engine (e.g. SQLite lacks certain `ALTER TABLE` capabilities), add a no-op placeholder in that engine's directory to keep the version sequences in sync:

```sql
-- Not applicable for SQLite — schema already contains this column.
select 1;
```

5. Commit the files. The next server start will apply them.

## SQLite-specific notes

- **Foreign keys** are enabled at the connection level (`foreign_keys = ON`), not in migrations, because SQLite pragmas cannot run inside a transaction.
- **`RETURNING`** is supported (SQLite ≥ 3.35). The existing migrations rely on it.
- **`ALTER TABLE`** in SQLite is limited. Adding columns is fine; dropping or renaming columns requires recreating the table. When a Postgres migration uses `ALTER TABLE … DROP COLUMN`, the SQLite counterpart usually needs a more involved table-rebuild approach.

## Environment variables

The active database engine is controlled by the `DATABASE` environment variable. Zealot will only run the migrations for the configured engine.

| `DATABASE` value | migration directory used |
|---|---|
| `postgres` (default) | `migrations/postgres/` |
| `sqlite` | `migrations/sqlite/` |
