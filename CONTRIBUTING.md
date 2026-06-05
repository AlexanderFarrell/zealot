# Contributing to Zealot

---

This guide covers how to write, review, and maintain Zealot documentation. Code contribution notes are at the end. There are two primary audiences: someone writing or editing a doc page, and a reviewer checking a PR that touches docs.

---

## Doc style guide

### Audience and tone

The primary reader is a self-hosting technical user comfortable with Docker and the command line. Write in second person — "you", not "the user" or "one". Use present tense throughout: "returns", "creates", not "will return", "will create". Technical but accessible: do not assume domain expertise beyond basic terminal use. No marketing language, no superlatives. No emoji in headings or body prose; emoji are acceptable only in ZealotScript syntax examples illustrating the `:shortcode:` feature.

### Document structure

Every file starts with `# Title`, then `---`, then an optional one-to-two sentence intro paragraph. Use `##` for primary sections, `###` for sub-sections, `####` only when a third level is genuinely required. One trailing blank line at end of file.

### Code blocks and commands

All shell commands go in a fenced code block. Use no language tag for plain shell commands — do not prefix commands with `$`. Use `bash` only when the command is clearly bash-specific. Use `json` for JSON examples, `lua` for Lua scripts. Output samples go in a bare fence or blockquote, clearly labelled:

```
# output
...
```

Multi-step sequences use a numbered list where each step contains a code block immediately below it.

### Tables

Use `| col | col |` syntax with a separator row `|---|---|`. Keep column headers short (one to three words). Introduce every table with a one-sentence paragraph. Prefer tables over bullet lists when there are two or more parallel attributes — for example, a list of fields with their types and descriptions.

### Links

Use relative links within `docs/`: `./data-model.md`, `./zealotscript/README.md`. From the repo root (for example, in this file): `./docs/data-model.md`. Link text should be the page title or a short descriptive phrase. Avoid "click here" or "this document".

### Terminology

The following table lists the canonical spellings. See [Glossary](./docs/glossary.md) for full definitions.

| Term | Use |
|---|---|
| item | lowercase |
| item type, type | lowercase |
| attribute kind | lowercase |
| ZealotScript | capital Z, capital S, one word |
| planner | lowercase |
| rules engine | lowercase |
| MCP | all caps |
| API key | "API" all caps, "key" lowercase |
| database | prefer "the database" or "the SQLite database" over "DB" in prose |

Prefer "item" over "record", "entry", or "node" except where those terms are exact (for example, "planner entry" is the correct term). Do not use "Zealot" as a verb.

---

## When to update docs

Any change that affects user-visible behaviour or developer-facing interfaces requires a corresponding doc update in the same PR. The following table maps change types to the files that need updating.

| Change type | Doc file(s) to update |
|---|---|
| New or changed HTTP endpoint or response shape | `docs/http-api.md` |
| New or changed attribute base type | `docs/data-model.md`, `docs/glossary.md` |
| New or changed item field, link relationship, or trigger kind | `docs/data-model.md` |
| New or changed Lua API function or `zealot.event` field | `docs/data-model.md` (Lua API summary), `docs/rules-engine.md` |
| New or changed MCP tool | `docs/mcp.md` |
| New or changed ZealotScript syntax | `docs/data-model.md` (ZealotScript section), `docs/zealotscript/README.md` |
| New environment variable or config option | `docs/deployment.md` |
| Changed build step or toolchain requirement | `docs/building.md` |
| Changed migration system behaviour | `docs/migrations.md` |
| New user-visible feature | `docs/overview.md` and the relevant feature doc |
| New domain term | `docs/glossary.md` |

If a change touches multiple files, list each in the PR description.

---

## PR review checklist

Run through this checklist before approving any PR that includes doc changes.

- [ ] All new terminology matches [docs/glossary.md](./docs/glossary.md), or the glossary has been updated.
- [ ] Code blocks are fenced with the correct language tag (or bare for shell commands).
- [ ] Commands have been validated against the current codebase (see [Command and example validation](#command-and-example-validation)).
- [ ] Links are relative, resolve correctly, and use descriptive text.
- [ ] Tables have an intro sentence.
- [ ] Screenshots, if added or changed, follow the [screenshot refresh policy](#screenshot-refresh-expectations).
- [ ] The "Entity Relationship Summary" in `docs/data-model.md` still accurately reflects the schema if any entity was added or changed.
- [ ] `docs/readme.md` is updated if a new doc page was added.
- [ ] `docs/glossary.md` is updated if a new domain term was introduced.
- [ ] Tone and voice match the style guide: second person, present tense, no marketing language.
- [ ] No broken links. Spot-check relative paths with:
  ```
  grep -roh '\](\.\/[^)]\+)' docs/ | grep -o '\.\/[^)]*' | sort -u
  ```

---

## Command and example validation

Every shell command, API call, and code example in the docs must be verifiable. Before merging, validate any example that you added or changed.

### Shell commands

Run the command against the Docker stack:

```
npm run docker
```

Paste the command from the doc into a terminal and confirm the output matches any described output. Note the stack version in the PR description for new commands. Pay particular attention to:

- `docker compose` subcommands — flags and service names change between Compose versions.
- `curl` invocations against the HTTP API.
- `cargo`, `npm`, and `rustup` commands in `docs/building.md`.

### HTTP API examples

All `curl` examples in `docs/http-api.md` should be validated against a local Zealot instance.

1. Start the stack: `npm run docker`
2. Register an account and generate an API key via the UI or `POST /auth/api_key`.
3. Run each changed `curl` command and verify the HTTP status code and response shape match the documented example.

For endpoints that require existing data (for example, `GET /item/{id}`), create minimal fixture data first.

### Lua rule scripts

For cookbook examples in `docs/rules-engine.md`:

1. Create a new rule and paste the script verbatim.
2. Set the trigger to Manual.
3. Click Run Now.
4. Confirm the output log matches the described behaviour and no Lua error appears.

If an example depends on instance-specific data (such as a type named "Task"), note that dependency in a comment at the top of the script.

### ZealotScript syntax examples

1. Create or open any item in the running Zealot UI.
2. Paste the example into the body.
3. Switch to preview mode and confirm the rendered output matches the intent described in the docs.

---

## Screenshot refresh expectations

### When to refresh

Refresh a screenshot when:

- The UI element it depicts has changed (layout, labels, or controls).
- A new feature is described and a screenshot would materially aid comprehension.
- A placeholder (see below) is resolved.

Do not replace a screenshot simply because the data in it changed (for example, different item titles). Replace it only when the structure or appearance has changed.

### Format and placement

- Format: PNG at 2× pixel density. Use browser DevTools device emulation at 2× DPR if needed.
- Dimensions: capture the minimum viewport that shows the relevant element with comfortable margin. Do not include the OS window chrome unless it adds context.
- File location: `docs/screenshots/<feature-area>-<descriptor>.png`. Examples: `planner-daily.png`, `item-wiki-page.png`, `settings-types.png`.
- Embed with: `![Alt text describing the screenshot](./screenshots/<filename>.png)`
- Alt text must describe what the screenshot shows in one sentence.

### Placeholders

When writing new docs that require a screenshot but the UI is not yet available, use this block:

```
> **Screenshot placeholder:** One-sentence description of what to capture.
> _Replace with `docs/screenshots/<filename>.png` once captured._
```

Remove the placeholder entirely when the real screenshot is added.

### How to take a screenshot

1. Start Zealot: `npm run docker`
2. Navigate to the relevant screen.
3. Use the browser's built-in screenshot tool or a capture utility at 2× DPR.
4. Crop to the relevant area. Annotate with arrows or highlights only if the focus would otherwise be ambiguous.
5. Place the file in `docs/screenshots/` and update the `![...]()` reference in the doc.

---

## Code contribution notes

For build instructions, see [Building & Running](./docs/building.md). For a codebase orientation, see [Architecture](./docs/architecture.md). Run `cargo test` for Rust unit tests and `npm test` for TypeScript tests. PRs should pass CI before review.
