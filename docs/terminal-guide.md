# Terminal Guide: CLI and TUI

This guide shows how to use Zealot from a terminal with the `zealot` CLI and the
full-screen `zealot-tui` interface. Both clients talk to the same Zealot server
as the web, desktop, and mobile apps. They share one config file, one login, and
the same item, planner, habit, rule, and media data.

Use the CLI when you want a quick command, a scriptable workflow, JSON output, or
an editor round trip. Use the TUI when you want to stay in a keyboard-driven
workspace for planning, browsing, search, habits, and rules.

---

## Prerequisites

You need:

- A running Zealot server.
- A Zealot account on that server.
- The terminal binaries built or installed locally.
- A Rust toolchain if you are building those binaries from source.

From a source checkout, build both terminal clients:

```bash
cargo build --release -p zealot-cli -p zealot-tui
```

This produces:

- `target/release/zealot`
- `target/release/zealot-tui`

If those binaries are on your `PATH`, the examples below work exactly as shown.
If not, run them by path, for example `target/release/zealot status`.

---

## First Login

Log in once with the CLI:

```bash
zealot login https://zealot.example.com
zealot status
```

`zealot login` prompts for your username and password, mints a dedicated API key,
and stores it in the terminal config file. The same stored key is used by both
`zealot` and `zealot-tui`.

For a local development server, use the local server URL:

```bash
zealot login http://localhost:8456
```

After login, verify the selected server and account:

```bash
zealot status
```

To revoke the stored API key and remove local credentials:

```bash
zealot logout
```

To forget local credentials but leave the API key active on the server:

```bash
zealot logout --keep-key
```

---

## Config and Profiles

The default config path is:

```text
~/.config/zealot/config.toml
```

The file is written with `0600` permissions on Unix because it contains API keys.
A typical config looks like this:

```toml
default_profile = "home"

[profiles.home]
server_url = "https://zealot.example.com"
api_key = "..."
api_key_id = 42
journal_item = "Journal"

[profiles.work]
server_url = "https://zealot.internal.example"
api_key = "..."
```

`journal_item` is optional. When set, `zealot journal`, `zealot j`, and the TUI
Today journal input add comments to that item. It can be an item title or a
numeric item id.

Select a profile per command:

```bash
zealot --profile work status
zealot --profile home day
```

Or set it for the shell session:

```bash
export ZEALOT_PROFILE=work
zealot status
```

Connection settings are resolved in this order:

| Source | Use case |
|---|---|
| `--url` and `--api-key` | One command with explicit credentials |
| `ZEALOT_URL` and `ZEALOT_API_KEY` | CI, scripts, temporary shells |
| `--profile` or `ZEALOT_PROFILE` | Choose one stored profile |
| `default_profile` | Normal interactive use |
| Sole profile in the config | Convenience fallback |

Override the config path with:

```bash
export ZEALOT_CONFIG=/path/to/config.toml
```

---

## Shared Terminal Conventions

### Item References

Most commands that take an item accept either a numeric id or an exact title:

```bash
zealot view 42
zealot view "Website Redesign"
```

If the argument parses as a positive integer, it is treated as an id. Otherwise,
it is treated as a title.

### Dates

Date arguments accept:

| Form | Meaning |
|---|---|
| `today`, `now` | Current local date |
| `yesterday`, `yest` | Previous day |
| `tomorrow`, `tom`, `tmrw` | Next day |
| `+3`, `-1` | Relative days from today |
| `mon`, `friday` | Upcoming weekday, including today |
| `YYYY-MM-DD` | Exact date |

Examples:

```bash
zealot day tomorrow
zealot habit done Exercise fri
zealot block ls 2026-07-10
```

### Time Ranges

Time block commands accept ranges such as:

```text
9:00-10:30
09:00-10:30
9-10
```

The end time must be after the start time.

### Attributes

Attribute values are usually passed as `KEY=VALUE`:

```bash
zealot add "Write launch notes" -t Task -A Status=Open -A Priority=2
zealot item attr set "Write launch notes" Due=2026-07-10 Done=false
```

Values are coerced automatically:

| Input | Stored as |
|---|---|
| `true`, `false` | Boolean |
| `3` | Integer |
| `3.5` | Decimal |
| `[a,b,c]` | List |
| Anything else | Text |

### Stdin and Editors

Commands that accept freeform text usually read stdin when text is omitted or
passed as `-`. `item new` reads piped stdin when `--content` is omitted:

```bash
git log --oneline -5 | zealot item append "Release Notes"
echo "Longer thought" | zealot journal
```

Commands with editor integration use `$VISUAL`, then `$EDITOR`, then `vi`:

```bash
zealot item edit "Reading List"
zealot item edit "Reading List" --full
zealot rule edit "Daily Cleanup"
```

`item edit --full` opens TOML frontmatter plus item content, so you can edit the
title, types, attributes, and body in one buffer.

### JSON, Color, and Exit Codes

Add `--json` to commands when scripting:

```bash
zealot --json filter 'Status=Open' | jq -r '.[].title'
```

Color is enabled only for interactive terminal output. Disable it explicitly with
`--no-color` or `NO_COLOR=1`.

Exit codes:

| Code | Meaning |
|---|---|
| `0` | Success |
| `1` | Runtime error |
| `2` | Usage or flag error |
| `3` | Not found |
| `4` | Not authenticated |

---

## CLI Basics

Run:

```bash
zealot --help
zealot item --help
zealot item edit --help
```

High-frequency commands have short aliases:

| Alias | Full command | Purpose |
|---|---|---|
| `zealot d` | `zealot day` | Day dashboard |
| `zealot s` | `zealot search` | Search items |
| `zealot v` | `zealot view` | View an item |
| `zealot a` | `zealot add` | Create an item |
| `zealot j` | `zealot journal` | Add a journal comment |

Launch the TUI from the CLI:

```bash
zealot tui
```

Or run it directly:

```bash
zealot-tui
```

---

## Daily Planning From the CLI

Show today's plan, habits, time blocks, and journal:

```bash
zealot day
zealot d
```

Show another day:

```bash
zealot day tomorrow
zealot day 2026-07-10
```

Include full item content in the day view:

```bash
zealot day --full
```

Show wider planner views:

```bash
zealot week
zealot week 2026-W28
zealot month 7 2026
zealot year 2026
```

---

## Items From the CLI

Create a quick item:

```bash
zealot add "Fix the boiler"
```

Create a structured item:

```bash
zealot item new "Write launch notes" \
  --type Task \
  --attr Status=Open \
  --attr Priority=2 \
  --parent "Website Redesign"
```

Create an item with inline content:

```bash
zealot item new "Meeting Notes" --content "Kickoff notes go here."
```

Create an item in your editor:

```bash
zealot item new "Research Notes" --edit
```

View an item:

```bash
zealot view "Research Notes"
zealot item view 42 --meta
zealot item view 42 --raw
zealot item view 42 --attrs
```

Edit content:

```bash
zealot item edit "Research Notes"
```

Edit title, types, attributes, and content together:

```bash
zealot item edit "Research Notes" --full
```

Append a log entry:

```bash
zealot item append "Release Notes" "Shipped terminal guide."
make test 2>&1 | tail -40 | zealot item append "Build Failures"
```

List and rediscover items:

```bash
zealot item ls
zealot item ls --type Project
zealot item recent --limit 20
zealot item random --count 5
zealot item top --limit 15
```

Explore relationships:

```bash
zealot item children "Website Redesign"
zealot item related "Website Redesign"
zealot item backlinks "Website Redesign"
```

Update attributes and types:

```bash
zealot item attr ls "Write launch notes"
zealot item attr set "Write launch notes" Status=Done
zealot item attr rm "Write launch notes" Priority
zealot item attr rename "Write launch notes" Due "Due Date"

zealot item type add "Write launch notes" Task
zealot item type rm "Write launch notes" Idea
```

Export an item:

```bash
zealot item export "Project Brief" --pdf
zealot item export "Project Brief" --docx --out project-brief.docx
```

Delete an item:

```bash
zealot item rm 42
zealot item rm 42 --yes
```

Rebuild the derived link index after large content changes or repair work:

```bash
zealot item rebuild-links
```

---

## Search and Filtering

Search titles:

```bash
zealot search boiler
zealot s boiler
```

Search content or headings:

```bash
zealot search "heat pump" --content
zealot search "weekly review" --heading
```

Use regex:

```bash
zealot search '^MMSS-[0-9]+' --regex
```

Control paging:

```bash
zealot search zealot --limit 50
zealot search zealot --offset 50
zealot search zealot --all
```

Filter by attribute expressions:

```bash
zealot filter 'Status=Open'
zealot filter 'Status=In Progress' 'Due<=2026-07-10'
zealot filter 'Priority>=3' 'Note~renovation'
```

Filter operators:

| Operator | Meaning |
|---|---|
| `=` | Equals |
| `!=` | Not equals |
| `>` | Greater than |
| `<` | Less than |
| `>=` | Greater than or equal |
| `<=` | Less than or equal |
| `~` | Case-insensitive contains |

---

## Habits and Repeats

List habits for today:

```bash
zealot habit ls
```

List another day:

```bash
zealot habit ls yesterday
```

Set a habit status:

```bash
zealot habit done Exercise
zealot habit skip Exercise today --comment "Rest day"
zealot habit alt Exercise fri --comment "Walked instead"
zealot habit undo Exercise
```

Show a 7-day grid ending today or another date:

```bash
zealot habit week
zealot habit week 2026-07-10
```

List all habit-tracked items:

```bash
zealot habit items
```

Habit statuses are:

| Status | Use |
|---|---|
| `Not Complete` | No completion recorded |
| `Complete` | Habit done |
| `Skip` | Intentionally skipped |
| `Alternate` | Satisfied by an alternate action |

---

## Time Blocks

List today's time blocks:

```bash
zealot block ls
```

Add a time block:

```bash
zealot block add "Deep Work" 9:00-10:30
zealot block add "Deep Work" 9:00-10:30 tomorrow --note "Thesis chapter"
```

Edit a block by id:

```bash
zealot block edit 12 --time 10:00-11:30
zealot block edit 12 --date tomorrow --note "Moved after standup"
```

Delete a block:

```bash
zealot block rm 12
```

---

## Journaling and Comments

Add a timestamped journal comment to your configured `journal_item`:

```bash
zealot journal "Feeling clear after the demo."
zealot j "Feeling clear after the demo."
```

Read journal text from stdin:

```bash
echo "Longer thought" | zealot journal
```

Add a comment to any item:

```bash
zealot comment add "Website Redesign" "Kickoff went well."
zealot comment add "Website Redesign" "Backfilled note." --at "2026-07-03 09:30:00"
```

List comments:

```bash
zealot comment ls --day
zealot comment ls --day yesterday
zealot comment ls --item "Website Redesign"
```

Edit or remove comments:

```bash
zealot comment edit 123 "Updated comment text"
zealot comment rm 123
```

---

## Types and Attributes

List and inspect item types:

```bash
zealot type ls
zealot type ls --counts
zealot type view Task
```

Create and remove item types:

```bash
zealot type new Recipe --description "Something to cook" --attr Ingredients --attr Serves
zealot type rm Recipe
zealot type rm Recipe --force
```

List and inspect attribute kinds:

```bash
zealot attr ls
zealot attr view Status
```

Create and remove attribute kinds:

```bash
zealot attr new Serves --base integer --description "Number of servings"
zealot attr new Status --base dropdown --config '{"values":["Open","Done"]}'
zealot attr rm Serves
zealot attr rm Serves --force
```

Supported attribute bases are:

```text
text, integer, decimal, date, week, dropdown, boolean, list, item
```

---

## Rules and Automation

List rules:

```bash
zealot rule ls
```

View a rule:

```bash
zealot rule view "Daily Cleanup"
zealot rule view "Daily Cleanup" --script
```

Run a rule immediately:

```bash
zealot rule run "Daily Cleanup"
```

Enable or disable a rule:

```bash
zealot rule enable "Daily Cleanup"
zealot rule disable "Daily Cleanup"
```

Edit the Lua script in your editor:

```bash
zealot rule edit "Daily Cleanup"
```

For the Lua API and rule design patterns, see [Rules Engine](./rules-engine.md).

---

## Media Files

List media files:

```bash
zealot media ls
zealot media ls screenshots/
```

Upload and download:

```bash
zealot media put diagram.png
zealot media put diagram.png screenshots
zealot media get screenshots/diagram.png --out /tmp/diagram.png
```

Manage folders and paths:

```bash
zealot media mkdir screenshots/archive
zealot media mv screenshots/old.png screenshots/archive/old.png
zealot media rm screenshots/archive/old.png
```

---

## Raw API and Shell Completions

Use `zealot api` for endpoints that are not wrapped by a first-class command:

```bash
zealot api GET /item/recent
zealot api GET '/item/search?term=rust&scope=content'
zealot api POST /rule/1/run
zealot api PATCH /item/42 --body '{"title":"New Title"}'
echo '{"title":"New Title"}' | zealot api PATCH /item/42 --body -
```

Generate shell completions:

```bash
zealot completions bash > ~/.local/share/bash-completion/completions/zealot
zealot completions zsh > ~/.zfunc/_zealot
zealot completions fish > ~/.config/fish/completions/zealot.fish
```

Supported completion shells are bash, zsh, fish, elvish, and powershell.

---

## TUI Basics

Launch the TUI:

```bash
zealot tui
```

Or:

```bash
zealot-tui
```

The TUI starts on the Today screen. It uses the same config resolution rules as
the CLI: profiles, environment variables, and `ZEALOT_CONFIG` all work the same
way.

Global keys:

| Key | Action |
|---|---|
| `1` | Today |
| `2` | Browse |
| `3` or `/` | Search |
| `4` | Habits |
| `5` | Rules |
| `:` or `Ctrl-P` | Command palette |
| `R` | Refresh current screen |
| `?` | Help overlay |
| `q` or `Esc` | Back or quit |
| `Ctrl-C` | Force quit |

The command palette lets you jump to screens, refresh, quit, and open an item by
title.

---

## Today in the TUI

Today shows the day plan, habits, journal comments, and time blocks for one
date. It auto-refreshes every 60 seconds while no journal input is active.

Keys:

| Key | Action |
|---|---|
| `[` | Previous day |
| `]` | Next day |
| `t` | Return to today |
| `j`, `Down` | Select next habit |
| `k`, `Up` | Select previous habit |
| `Space` | Cycle selected habit status |
| `Enter` | Open selected habit's item |
| `i` | Start a journal entry |
| `Esc` while typing | Cancel journal entry |
| `Enter` while typing | Submit journal entry |

Habit status cycling uses:

```text
Not Complete -> Complete -> Skip -> Alternate -> Not Complete
```

The journal input requires `journal_item` in the selected config profile.

---

## Browse, Search, and Viewer in the TUI

Browse is a lazy-loaded item tree. It starts with root items and loads children
when you expand a node.

Browse keys:

| Key | Action |
|---|---|
| `j`, `Down` | Move down |
| `k`, `Up` | Move up |
| `g` | Top |
| `G` | Bottom |
| `l`, `Right` | Expand or load children |
| `h`, `Left` | Collapse |
| `Enter` | Open selected item |
| `e` | Edit selected item content |

Search is live and debounced. Type a query and results update automatically.

Search keys:

| Key | Action |
|---|---|
| Type text | Update query |
| `Backspace` | Delete character |
| `Tab` | Cycle title, content, and heading search |
| `Ctrl-R` | Toggle regex |
| `Down`, `Up` | Select result |
| `Enter` | Open selected result |
| `Esc` | Return to Today |

The viewer shows rendered ZealotScript and item metadata.

Viewer keys:

| Key | Action |
|---|---|
| `j`, `Down` | Scroll down |
| `k`, `Up` | Scroll up |
| `Ctrl-D` | Page down |
| `Ctrl-U` | Page up |
| `g` | Top |
| `e` | Edit item content |
| `b` | Show backlinks |
| `r` | Show related items |
| `c` | Show children |
| `Backspace`, `Ctrl-O`, `q`, `Esc` | Back |

When a backlinks, related, or children overlay is open, use `j`/`k` or arrow keys
to select an item, `Enter` to open it, and `Esc` to close the overlay.

---

## Habits and Rules in the TUI

The Habits screen shows a two-week grid of habit entries.

Habits keys:

| Key | Action |
|---|---|
| `h`, `Left` | Move left |
| `l`, `Right` | Move right |
| `j`, `Down` | Move down |
| `k`, `Up` | Move up |
| `Space` | Cycle selected cell status |
| `[` | Shift one week earlier |
| `]` | Shift one week later |
| `t` | Return grid end date to today |

The Rules screen lists automation rules with status and last-run details.

Rules keys:

| Key | Action |
|---|---|
| `j`, `Down` | Select next rule |
| `k`, `Up` | Select previous rule |
| `r`, `Enter` | Run selected rule |
| `e` | Edit selected rule script |

---

## Editor Round Trips in the TUI

Pressing `e` in Browse, Viewer, or Rules temporarily leaves the full-screen
terminal, opens your editor, and then restores the TUI after you save and exit.

The editor command is selected in this order:

```text
$VISUAL, $EDITOR, vi
```

If you close the editor without changes, nothing is sent to the server. If you
save changes, the TUI updates the item content or rule script and refreshes the
visible screen.

---

## Common Workflows

### Start the Day

```bash
zealot day
zealot habit done Shower
zealot block add "Deep Work" 9:00-10:30 --note "Most important task"
zealot tui
```

Use the CLI to set up the day quickly, then use the TUI Today screen to adjust
habits and inspect the plan.

### Capture a Task Without Breaking Flow

```bash
zealot add "Email Sam about contract" -t Task -A Status=Open -A Priority=2
```

Or capture richer content from another command:

```bash
pbpaste | zealot item new "Draft notes from clipboard"
```

On Linux, replace `pbpaste` with your clipboard command, or pipe any command that
prints text.

### Review Open Work

```bash
zealot filter 'Status=Open' 'Priority>=2'
zealot --json filter 'Status=Open' | jq -r '.[].title'
```

Open the TUI Search screen with `/` when you want to inspect and jump between the
results interactively.

### Weekly Review

```bash
zealot week
zealot habit week
for d in -6 -5 -4 -3 -2 -1 today; do
  zealot --json comment ls --day "$d"
done | jq -rs 'add[] | "\(.timestamp)  \(.content)"'
```

### Maintain Automation

```bash
zealot rule ls
zealot rule run "Weekly Review"
zealot rule edit "Weekly Review"
```

Use the TUI Rules screen when you want to run several rules and inspect their
last outputs without leaving the terminal workspace.

---

## Troubleshooting

`error: ... not authenticated`

Run:

```bash
zealot login https://your-server
```

Or check that `ZEALOT_URL` and `ZEALOT_API_KEY` are both set when using env-based
authentication.

`no credentials - run zealot login first`

The TUI could not resolve credentials. Run `zealot status` first; it reports the
same profile and config state that the TUI uses.

`no journal_item configured`

Add `journal_item` to the active config profile:

```toml
[profiles.home]
journal_item = "Journal"
```

Then make sure an item with that title or id exists.

An item title is not found

Use the numeric item id if titles are duplicated or if you are unsure of the
exact spelling:

```bash
zealot search "partial title"
zealot view 42
```

TUI display looks broken

Use a modern terminal emulator, make sure the terminal size is large enough, and
try disabling shell prompt integrations that alter alternate-screen programs. If
the shell remains in raw mode after a crash, run:

```bash
reset
```

Server or API errors

Check the server is running, then use:

```bash
zealot status
zealot api GET /health
```

For broader operational help, see [Troubleshooting and Recovery](./troubleshooting.md).

---

## Related Documentation

- [CLI Reference](./cli.md) covers the command tree and examples in a compact
  reference format.
- [TUI Reference](./tui.md) covers screens, keys, and implementation notes.
- [Quickstart](./quickstart.md) covers first-time Zealot setup.
- [Data Model](./data-model.md) explains items, types, attributes, links,
  repeats, comments, rules, and ZealotScript.
- [Rules Engine](./rules-engine.md) explains Lua automation.
- [HTTP API](./http-api.md) documents the REST API behind the terminal clients.
