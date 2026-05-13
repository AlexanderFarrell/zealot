# ZealotScript — Emoji Shortcodes

Emoji can be inserted using `:shortcode:` syntax — colon-delimited identifiers that expand
to their unicode character at parse time.

## Syntax

```
I love :pizza: and :coffee: every morning. :rocket:
```

Renders as: `I love 🍕 and ☕ every morning. 🚀`

**Rules:**
- Wrap the shortcode in colons with no spaces: `:smile:` ✓, `: smile :` ✗
- Only lowercase letters, digits, underscores, `+`, and `-` are valid inside the colons
- Unknown shortcodes are left as literal text (including the colons): `:notanemoji:` stays `:notanemoji:`
- The document stores the resolved unicode character, not the shortcode

## Serializer behavior

The serializer writes the emoji character as-is. Emoji entered directly (e.g. copy-pasted
from another source) also round-trips cleanly — shortcode encoding is not required.

## Live editor behavior

### Auto-replace on completion

When you finish typing `:shortcode:` in the editor (the closing `:` triggers replacement),
the shortcode is immediately replaced with the unicode character in-line.

### Picker as you type

After typing `:` followed by any characters, a suggestion popup appears showing up to 8
matching shortcodes with their emoji previews. Click a suggestion (or press Escape to dismiss)
to insert it at the cursor position.

## Common shortcodes quick-reference

| Shortcode | Emoji | | Shortcode | Emoji |
|---|---|---|---|---|
| `:smile:` | 😄 | | `:thumbsup:` | 👍 |
| `:heart:` | ❤️ | | `:fire:` | 🔥 |
| `:rocket:` | 🚀 | | `:check:` | ✅ |
| `:x:` | ❌ | | `:warning:` | ⚠️ |
| `:bug:` | 🐛 | | `:star:` | ⭐ |
| `:tada:` | 🎉 | | `:bulb:` | 💡 |
| `:eyes:` | 👀 | | `:pray:` | 🙏 |
| `:clap:` | 👏 | | `:muscle:` | 💪 |
| `:coffee:` | ☕ | | `:pizza:` | 🍕 |
| `:cat:` | 🐱 | | `:dog:` | 🐶 |
| `:zap:` | ⚡ | | `:sparkles:` | ✨ |
| `:100:` | 💯 | | `:memo:` | 📝 |

The full shortcode list covers ~500 entries spanning smileys, people, animals, food, travel,
nature, objects, and symbols.

## Implementation notes

- **Shortcode map**: `packages/ui/src/zealotscript/emoji_map.ts` — `Record<string, string>` exported as `EMOJI_MAP`, with a `lookupEmoji(shortcode)` helper
- **Parser expansion**: `packages/ui/src/zealotscript/parse/parse_inline.ts` — `:shortcode:` rule runs before backtick code; resolves via `lookupEmoji`, falls through to literal text if unknown
- **Input rule**: `packages/ui/src/zealotscript/zealotscript_editor.ts` — `/:([a-z0-9_+\-]+):$/` replaces matched text with the emoji character as soon as the closing `:` is typed
- **Picker**: `ZealotScriptEditor._showEmojiPicker` / `_updateEmojiPickerContent` — detects `:query` pattern after each transaction, shows filtered suggestions; dismissed on selection, Escape, or click-outside
- **CSS**: `packages/content/src/css/tags.scss` — `.zealotscript-emoji-picker`, `.zealotscript-emoji-option`
- **No new schema node** — resolved emoji is stored as a plain text node
