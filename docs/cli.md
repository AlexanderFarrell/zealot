# Zealot CLI (`zealot`)

Operate your entire wiki, planner, and habit tracker from the terminal. The CLI is a
first-class client — it authenticates like the desktop and mobile apps (username +
password mints an API key) and talks to the same REST API.

For a start-to-finish user guide covering both terminal clients, see
[Terminal Guide: CLI and TUI](./terminal-guide.md). This page is the compact CLI
reference.

```
$ zealot d
FRIDAY 2026-07-03 (TODAY)

Time blocks
   9:00–10:30  [#12]  Deep Work — thesis chapter

Plan
  #3672 Setup Geoserver locally
  #3688 Development Systems Assessment

Habits
  ✔ 🚿 Shower
  · ✍️ Journal
  2/15 complete

Journal
  07:14 ❤️ Health Slept well, feeling sharp.
```

---

## Install & connect

```bash
cargo build --release -p zealot-cli     # produces target/release/zealot
zealot login https://zealot.example.com # prompts for username + password
zealot status                           # verify the connection
```

`login` mints a dedicated API key (labelled `CLI (hostname)`) and stores it in
`~/.config/zealot/config.toml` with `0600` permissions. `zealot logout` revokes the
key server-side and forgets it. The same config powers the TUI (`zealot-tui`) —
log in once, use both.

### Config file

```toml
default_profile = "home"

[profiles.home]
server_url   = "https://zealot.example.com"
api_key      = "…"
api_key_id   = 42
journal_item = "Journal"   # target of `zealot j` (title or numeric id)

[profiles.work]
server_url = "https://zealot.internal.corp"
api_key    = "…"
```

| Setting source | Precedence |
|---|---|
| `ZEALOT_URL` + `ZEALOT_API_KEY` env vars | highest — no config file needed (CI, scripts) |
| `--profile NAME` flag / `ZEALOT_PROFILE` | selects a profile |
| `default_profile` in the config | fallback |
| `ZEALOT_CONFIG` | overrides the config file path |

---

## Everyday commands

Every `<ITEM>` argument accepts a numeric id **or** a title: `zealot v 42` and
`zealot v "Reading List"` both work.

Every `DATE` argument accepts `today` (the default), `yesterday`, `tomorrow`,
weekday names (`fri` = the upcoming Friday), relative offsets (`+3`, `-1`), or
`YYYY-MM-DD`.

| Shortcut | Full form | Does |
|---|---|---|
| `zealot d [DATE]` | `day` | Day dashboard: plan + habits + time blocks + journal |
| `zealot s TERM` | `search` | Search titles (`-c` content, `--heading`, `-r` regex) |
| `zealot v ITEM` | `view` / `item view` | Show an item, ZealotScript rendered for the terminal |
| `zealot a TITLE` | `add` / `item new` | Quick-add an item |
| `zealot j TEXT` | `journal` | One-line journal entry, timestamped now |

### Items

```bash
zealot item view "Reading List"            # rendered; --raw for source, --meta for everything
zealot item new "Fix the boiler" -t Task -A Status=Open -A Priority=2 --parent House
zealot item edit "Reading List"            # content in $EDITOR
zealot item edit "Reading List" --full     # title/types/attributes as TOML frontmatter + content
zealot item append "Standup Log" "- shipped the CLI"
git log --oneline -5 | zealot item append "Release Notes"   # stdin works everywhere
zealot item ls -t Project                  # items by type; also: recent, random, top
zealot item children House                 # also: related, backlinks
zealot item attr set 42 Status=Done        # also: attr rm / attr rename / attr (list)
zealot item type add 42 Task               # assign / remove types
zealot item export "Thesis" --pdf          # or --docx; -o FILE to name it
zealot item rm 42                          # confirms; --yes to skip
```

`--full` editing shows the item as TOML frontmatter + content in one buffer:

```
+++
title = "Reading List"
types = ["Collection"]

[attributes]
Status = "Active"
+++
# Books
- [ ] …
```

Change anything and save — title, types, and attributes are diffed and applied.
If the frontmatter fails to parse, the editor reopens with the error as a comment
at the top; your edit is never lost.

### Search & filter

```bash
zealot s "boiler"                    # title search (default)
zealot s "heat pump" -c -n 50        # content search, 50 results
zealot s '^MMSS-\d+' -r              # regex
zealot s zealot --all                # fetch every page
zealot filter 'Status=In Progress' 'Due<=2026-07-10'
zealot filter 'Priority>=3' 'Note~renovation'    # ~ is case-insensitive contains
```

Filter operators: `=` `!=` `>` `<` `>=` `<=` and `~` (ilike). Values are typed
automatically (`true`/`false` → boolean, numbers → numeric, `[a,b]` → list).

### Planner, habits, time blocks

```bash
zealot day tomorrow          # any date form works
zealot week                  # this ISO week; or `zealot week 2026-W28`
zealot month 8               # August this year
zealot habit ls              # today's habits with ✔ ↷ ◆ · glyphs
zealot habit done Exercise   # also: skip, alt, undo; -m "comment"
zealot habit done Exercise fri -m "5k run"
zealot habit week            # 7-day streak grid
zealot block add "Deep Work" 9:00-10:30 tomorrow -m "thesis"
zealot block ls              # today's timeline
```

### Journaling & comments

```bash
zealot j "Feeling great after the demo"    # → journal_item, timestamped now
echo "long thought" | zealot j             # stdin
zealot comment add "Project X" "kickoff went well"
zealot comment ls --day yesterday
zealot comment ls --item "Project X"
```

### Types, attributes, rules, media

```bash
zealot type ls --counts
zealot type new Recipe -d "Something to cook" -A Ingredients -A Serves
zealot attr new Serves --base integer --config '{"min":1}'
zealot rule ls                       # trigger, enabled, last run at a glance
zealot rule run Playground           # prints output/error; exit 1 on failure
zealot rule edit Playground          # Lua script in $EDITOR
zealot media ls screenshots/
zealot media put diagram.png screenshots
zealot media get screenshots/diagram.png -o /tmp/d.png
```

---

### Statistics

    zealot statistic items
    zealot statistic items --parent Health
    zealot statistic record "Body Weight" 81.4 --comment "Morning"
    zealot statistic entries "Body Weight" --limit 20
    zealot statistic daily "Body Weight" --start 2026-08-01T00:00:00Z
    zealot statistic summary "Body Weight" --start 2026-08-01T00:00:00Z --end 2026-09-01T00:00:00Z
    zealot statistic edit 42 --value 81.2
    zealot statistic rm 42

Times use RFC 3339. The global JSON flag emits the API DTOs.

---

## Scripting cookbook

`--json` on any command emits the raw API DTOs — pipe to `jq` and go.

```bash
# Titles of everything still open
zealot --json filter 'Status=Open' | jq -r '.[].title'

# Did I complete all habits today? (exit code for cron/scripts)
zealot --json habit ls | jq -e 'all(.status == "Complete")' >/dev/null

# Weekly review: dump this week's journal into a file
for d in -6 -5 -4 -3 -2 -1 today; do zealot --json comment ls --day $d; done \
  | jq -rs 'add[] | "\(.timestamp)  \(.content)"' > week-review.txt

# Capture a failing build into the wiki
make 2>&1 | tail -20 | zealot item append "Build Failures"

# Raw API escape hatch for anything not wrapped yet
zealot api GET '/item/search?term=rust&scope=content'
zealot api POST /rule/1/run
```

Output degrades automatically when piped: colors off, tables become aligned plain
text. `NO_COLOR` and `--no-color` are honored.

### Exit codes

| Code | Meaning |
|---|---|
| 0 | success |
| 1 | error (server error, failed rule, …) |
| 2 | usage error (bad flags) |
| 3 | not found |
| 4 | not authenticated — run `zealot login` |

### Shell completions

```bash
zealot completions zsh > ~/.zfunc/_zealot       # also bash, fish, elvish, powershell
```

---

## Command reference

Run `zealot --help` or `zealot <command> --help` — every command and flag is
documented inline. Top-level commands:

`login` `logout` `status` · `item` `view` `add` `search` `filter` ·
`day` `week` `month` `year` · `habit` `block` `comment` `journal` ·
`type` `attr` `rule` `media` · `api` `tui` `completions`
