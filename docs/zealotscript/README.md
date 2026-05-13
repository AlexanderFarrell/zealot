# ZealotScript — User Guide

ZealotScript is the lightweight markup language used for all wiki content in Zealot. It is a
superset of Markdown, extended with fenced-block syntax (`:::keyword … :::`) for richer
structured content.

## Quick reference

| Feature | Syntax |
|---|---|
| **Headings** | `# H1` … `###### H6` |
| **Bold** | `**text**` |
| **Italic** | `*text*` |
| **Strikethrough** | `~~text~~` |
| **Underline** | `_text_` |
| **Inline code** | `` `code` `` |
| **Highlight** | `<mark>text</mark>` |
| **Subscript** | `<sub>text</sub>` |
| **Superscript** | `<sup>text</sup>` |
| **Hard break** | `<br>` |
| **Link** | `[label](url)` |
| **Wiki link** | `[[Item Name]]` |
| **Type link** | `[[type:TypeName]]` |
| **Blockquote** | `> text` |
| **Bullet list** | `- item` |
| **Ordered list** | `1. item` |
| **Code block** | ` ```lang … ``` ` |
| **Table** | Standard Markdown pipes |
| **Admonitions** | `:::note … :::` |
| **YouTube embed** | `:::youtube VIDEO_ID` |
| **Inline math** | `$E = mc^2$` |
| **Math block** | `:::math … :::` |
| **Emoji** | `:smile:` → 😄 |

## Admonition kinds

`note` · `warning` · `danger` · `tip` · `info` · `success` · `important` · `caution` ·
`example` · `faq` · `todo`

## Guide pages

- [Math (LaTeX)](./math.md) — inline `$…$` and block `:::math` expressions
- [Emoji shortcodes](./emoji.md) — `:smile:` shortcode expansion, live picker, full reference

## Architecture reference

For implementation details see the source in:
- `packages/ui/src/zealotscript/` — parser, serializer, editor, viewer
- `packages/content/src/css/tags.scss` — all ZealotScript CSS classes
