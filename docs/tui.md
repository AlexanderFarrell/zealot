# Zealot TUI (`zealot-tui`)

A full-screen terminal workstation for Zealot: your day plan, habits, wiki, search,
and automations in one keyboard-driven interface built with
[ratatui](https://ratatui.rs).

```
┌ Friday 2026-07-03 (today) ─────────────────────────────────────────┐
│┌ Plan ──────────────────────┐┌ Habits 2/15 (Space to cycle) ──────┐│
││#3672 Setup Geoserver       ││✔ 🚿 Shower                         ││
││#3688 Systems Assessment    ││· ✍️ Journal                        ││
│└────────────────────────────┘└────────────────────────────────────┘│
│┌ Journal ───────────────────┐┌ Time blocks ───────────────────────┐│
││07:14 ❤️ Health Slept well. ││ 9:00–10:30 Deep Work — thesis      ││
│└────────────────────────────┘└────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
 Today  home · https://zealot.example.com   1-5 screens  / search  ? help
```

## Setup

The TUI shares credentials with the CLI — no separate setup:

```bash
cargo build --release -p zealot-tui
zealot login https://zealot.example.com   # once, via the CLI
zealot-tui
```

`ZEALOT_URL`/`ZEALOT_API_KEY`, `ZEALOT_PROFILE`, and `ZEALOT_CONFIG` env vars are
honored exactly like the CLI. The journal input uses the profile's `journal_item`.

## Screens

Switch with `1`–`5` (the Item viewer opens from Browse/Search), or the command
palette.

| # | Screen | What it does |
|---|---|---|
| 1 | **Today** | Day plan, habits, time blocks, and journal for any date. Cycle habit statuses in place, journal without leaving the dashboard. Auto-refreshes every 60s. |
| 2 | **Browse** | Lazily loaded item tree (roots → children) with a live ZealotScript preview pane. |
| 3 | **Search** | Search-as-you-type (250ms debounce) across titles, content, or headings, with regex support and match snippets. |
| — | **Item viewer** | Rendered ZealotScript with the attribute strip up top. Jump through backlinks, related items, and children; edit in `$EDITOR`. |
| 4 | **Habits** | Two-week grid of every habit. Move with `hjkl`, cycle any cell's status with `Space`, page through history with `[` `]`. |
| 5 | **Rules** | Automation list with trigger + last run; detail pane shows last output/error. Run rules in place, edit the Lua script in `$EDITOR`. |

## Keys

Global:

| Key | Action |
|---|---|
| `1`–`5` | switch screen |
| `/` | jump to Search |
| `:` or `Ctrl-P` | command palette (actions + open item by title) |
| `R` | refresh current screen |
| `?` | help overlay |
| `q` / `Esc` | back (viewer) or quit · `Ctrl-C` force quit |

Per screen:

| Screen | Keys |
|---|---|
| Today | `[` `]` prev/next day · `t` today · `j`/`k` select habit · `Space` cycle status · `Enter` open habit item · `i` journal (Enter submits, Esc cancels) |
| Browse | `j`/`k` move · `l` expand · `h` collapse · `g`/`G` top/bottom · `Enter` open · `e` edit |
| Search | type to search · `Tab` cycle scope · `Ctrl-R` regex · `↑`/`↓` select · `Enter` open |
| Viewer | `j`/`k`/`Ctrl-D`/`Ctrl-U`/`g` scroll · `e` edit · `b` backlinks · `r` related · `c` children · `Backspace`/`Ctrl-O` back |
| Habits | `hjkl` move in grid · `Space` cycle cell · `[` `]` shift week · `t` back to today |
| Rules | `j`/`k` select · `r`/`Enter` run · `e` edit script |

Habit statuses cycle `·` not complete → `✔` complete → `↷` skip → `◆` alternate.

## Editor integration

`e` anywhere suspends the TUI, opens `$VISUAL`/`$EDITOR`/`vi` on the item content
(or rule script), and resumes on save. No change → nothing sent. The terminal is
restored properly even on panic.

## Architecture notes

- `apps/tui/src/app.rs` — all state + key handling (Elm-style update)
- `apps/tui/src/net.rs` — every fetch/mutation is a spawned tokio task reporting
  back over one channel; stale responses are dropped via generation counters
- `apps/tui/src/ui/screens/` — one draw module per screen
- ZealotScript rendering comes from the shared `crates/zealot-zscript` renderer
  (same output as the CLI), parsed into ratatui spans via `ansi-to-tui`
- No SSE/WebSocket exists in the API, so the Today screen refreshes on a timer;
  every mutation triggers an immediate refetch of the visible screen
