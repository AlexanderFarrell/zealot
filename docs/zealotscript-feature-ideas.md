# ZealotScript Feature Ideas

## What ZealotScript already has

**Block nodes:** headings (h1-h6), paragraphs, bullet/ordered lists, code blocks (with language), blockquotes, tables (markdown + `:::table`), admonitions (`:::note/warning/danger/tip/info/success/important/caution/example/faq/todo`), YouTube embeds (`:::youtube`)

**Inline marks:** bold, italic, strikethrough, underline, code, highlight, subscript, superscript, links, wikilinks (`[[item]]`, `[[type:T]]`), hard breaks

---

## New things that would help a personal wiki

### Inline

| Feature | Syntax idea | Notes |
|---|---|---|
| Inline math | `$E = mc^2$` | Pairs with a math block for formulas |
| Keyboard shortcut | `<kbd>Ctrl+S</kbd>` | Renders as a key chip |
| Emoji shortcodes | `:smile:` | Avoids raw unicode in source |
| Mention / person | `@Alice` or `[[person:Alice]]` | Links to a person item type |
| Date reference | `@2025-06-01` | Links or highlights a date; useful for planner |
| Inline tag | `#project/zealot` | Quick tagging without leaving text flow |
| Abbreviation tooltip | `HTML` where `HTML` is defined elsewhere | Hover shows expansion |
| Color span | `<color:red>text</color>` | Subtle text color for emphasis |
| Spoiler | `\|\|hidden text\|\|` | Reveal on click/hover |
| Footnote reference | `[^1]` | Paired with a footnote block below |

---

### Block

| Feature | Syntax idea | Notes |
|---|---|---|
| Task list / checklist | `- [ ] task` / `- [x] done` | The single most requested wiki feature; integrates with planner |
| Horizontal rule | `---` on its own line | Near-standard, currently missing |
| Math block | ` ```math ` or `:::math` | Block equations via KaTeX/MathJax |
| Mermaid diagram | ` ```mermaid ` | Flowcharts, sequence, Gantt, ER, mindmap |
| Collapsible / details | `:::details Summary text` | Hide long tangents or spoilers |
| Definition list | `term\n: definition` | Glossaries, field definitions |
| Footnote definitions | `[^1]: Full footnote text` | Paired with inline refs |
| Image embed with caption | `![alt](url "caption")` | Needs a dedicated node for proper rendering |
| Properties / frontmatter | `:::properties` block | Key-value metadata (status, due date, priority) visible in-page |
| Transclusion | `![[other item name]]` | Embed another item's content inline |
| Columns layout | `:::columns` wrapping sub-blocks | Side-by-side content |
| Tabs | `:::tabs` with `:::tab Title` children | Alternate views of related content |
| Timeline | `:::timeline` with date+event children | Log of events, changelogs, history |
| Progress bar | `:::progress 7/10` or `70%` | Goal tracking at a glance |
| Rating | `:::rating 4/5` | Books, films, tools, experiences |
| Spoiler block | `:::spoiler` | Collapse sensitive or long content |
| Diff block | ` ```diff ` (special highlight) | Parsed as code but with +/- coloring |
| Audio embed | `:::audio <url>` | Podcasts, voice memos |
| Map pin | `:::map 51.5,-0.1` | Location notes |
| Template / snippet | `:::template` | Reusable block inserted by name |

---

### Wiki-specific linking & structure

| Feature | Notes |
|---|---|
| Heading anchors | `[[item#Section Title]]` — link to a specific heading within an item |
| Item alias | `[[long title\|short label]]` — display text differs from target |
| Backlinks panel | Shown below the editor; surfaces all `[[...]]` references to this item |
| Item transclusion preview | Hover a `[[link]]` to preview the target item |
| Auto-link known item names | Optionally linkify bare item names as you type |
| Broken link indicator | Visual indicator when a `[[wikilink]]` target doesn't exist |

---

### Planner / personal productivity

| Feature | Syntax / UI idea | Notes |
|---|---|---|
| Due date inline | `@due:2025-06-01` | Appears in item metadata, drives planner views |
| Recurring marker | `@repeat:weekly` | Flags an item or task as recurring |
| Priority inline | `@p1` / `@priority:high` | Filters and sorts in list views |
| Time estimate | `@est:2h` | Planning and time blocking |
| Status marker | `@status:draft/in-progress/done` | Page-level state beyond a tag |
| Habit tracker table | Special table variant with date columns | Streaks, checkboxes per day |
| Person/attendee list | `:::attendees` block | Meeting notes pattern |
| Action items extraction | Items marked `- [ ]` get surfaced to a task view | Cross-item task aggregation |

---

### Code / technical notes

| Feature | Notes |
|---|---|
| Numbered line highlights | ` ```ts {3,7-10} ` — highlight specific lines in code blocks |
| Filename label on code block | ` ```ts filename="main.ts" ` |
| Output block | ` ```output ` — visually distinct from source, no syntax highlight |
| HTTP request block | ` ```http ` — renders method, URL, headers nicely |
| JSON/YAML formatted display | Already works as code, but could get fold/collapse |

---

### Quality-of-life editor features

- Slash command menu (`/`) — type `/` to insert any block type by name
- Drag handles on block nodes to reorder
- Block ID anchors (stable `#id` for deep links that survive heading renames)
- Word/character count in toolbar
- Focus/zen mode (hide everything but the editor)
- Full-text search across all items surfacing heading context

---

## Priority picks

The highest-value gaps relative to what's already there:

1. **Task lists** — most impactful for the planner use case
2. **Mermaid diagrams** — biggest unlock for a dev/technical wiki
3. **Collapsible sections** — reduces visual noise in long pages
4. **Transclusion** (`![[item]]`) — the defining feature of a wiki vs. a flat editor
5. **Properties/frontmatter** — structured metadata without leaving the doc
