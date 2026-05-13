# TASK-022: ZealotScript — Emoji Shortcodes

## Context
Typing `:smile:` is more readable in source than a raw unicode emoji and avoids encoding issues
in plain-text storage. This is a purely inline feature with no new block nodes.

## Goal
Expand `:shortcode:` sequences to their corresponding unicode emoji during parsing, so the
document stores the resolved character and the source text round-trips cleanly.

## Requirements
- Syntax: `:shortcode:` — colon-delimited identifier, e.g. `:smile:`, `:rocket:`, `:check:`
- Resolved at **parse time** to the unicode character; the ProseMirror document stores the
  emoji text, not the shortcode
- Serializer: write the emoji character as-is (no re-encoding to shortcode) — emoji in source
  is valid ZealotScript
- Shortcode list: use the **`emoji-toolkit`** or **`@emoji-mart/data`** dataset, or ship a
  curated built-in map of ~500 common shortcodes (starter list: Github emoji list)
- Unknown shortcodes (`:notanemoji:`) are left as literal text including the colons
- Input rule in the editor: when the user types `:smile:` and presses Space or Enter,
  auto-replace with the unicode character inline
- shows selector as you type of options

## Implementation Notes
- Inline parser: add a regex pass in `parse_inline.ts` — `/:([a-z0-9_+-]+):/g` — look up in
  the emoji map, replace with a text node if found, leave as-is if not
- Emoji map can be a static `Record<string, string>` imported from a small module under
  `packages/ui/src/zealotscript/emoji_map.ts`
- No new schema node is needed — resolved emoji is just a text node
- ProseMirror input rule: use `InputRule` from `prosemirror-inputrules` to trigger replacement
  as the user types

## Dependencies
- TASK-017 (inline parser)

## Files Likely Involved
- `packages/ui/src/zealotscript/emoji_map.ts` — new file: `Record<string, string>`
- `packages/ui/src/zealotscript/parse/parse_inline.ts` — shortcode expansion rule
- `packages/ui/src/zealotscript/zealotscript_editor.ts` — InputRule for live replacement
