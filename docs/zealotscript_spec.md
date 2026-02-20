# ZealotScript Specification

Status: Draft (implementation-aligned)

This document defines ZealotScript as currently implemented by the parser/serializer/editor in `client/src/features/zealotscript`.

## 1. Document Model

A ZealotScript document is a sequence of block nodes separated by newlines.

Supported block node kinds:
- Paragraph
- Heading (`h1`–`h6`)
- Blockquote
- Bullet list
- Ordered list
- Fenced code block
- Table
- Admonition (`note`, `warning`, `danger`, `tip`, `info`, `success`, `important`, `caution`, `example`, `faq`, `todo`)

## 2. Block Syntax

### 2.1 Headings

Syntax:
- `# Heading 1`
- `## Heading 2`
- ... up to `###### Heading 6`

Rules:
- Must begin with `#{1,6}` followed by at least one space.
- Heading content supports full inline syntax.

### 2.2 Paragraphs

Any non-empty line not matched by a higher-priority block parser becomes a paragraph.

Rules:
- Blank lines create paragraph separation.
- Inline syntax is parsed inside paragraph text.

### 2.3 Blockquotes

Syntax:
- `> quoted line`
- `> another line`

Rules:
- Consecutive `>` lines are grouped into one blockquote.
- Leading whitespace before `>` is allowed.
- Inside the blockquote, paragraph splitting is preserved.
- On serialization, each quote line is prefixed with `>`.
- Blockquotes are round-trip safe (editor blockquote node <-> `>` syntax).

### 2.4 Lists

Syntax:
- Bullet: `- item` or `* item`
- Ordered: `1. item`

Rules:
- Nested lists use indentation (tab or 4-space units).
- List item text supports inline syntax.

### 2.5 Fenced Code Blocks

Syntax:
```
```
code here
```
```

Rules:
- Triple backticks open and close.
- Language tags after opening fence are currently ignored.

### 2.6 Tables

ZealotScript supports two table forms.

#### Markdown table

Syntax:
```
| Col A | Col B |
| --- | --- |
| A1 | B1 |
| A2 | B2 |
```

Rules:
- Header row + separator row required.
- Blank lines between header/separator/body rows are tolerated.
- Body rows are normalized to header column count.
- Inline syntax is parsed in cells.
- This supports pasted ChatGPT Deep Research markdown tables that include extra blank lines.

#### Command table

Syntax:
```
:::table
A | B | C
1 | 2 | 3
:::
```

Rules:
- `:::table` starts, `:::` ends.
- If second row is a markdown separator (`---` style), first row becomes header.
- Inline syntax is parsed in cells.

Serialization behavior:
- If first row is header cells, serializer emits markdown table.
- Otherwise serializer emits `:::table` form.

### 2.7 Admonitions

Syntax:
```
:::note
content
:::
```

Kinds:
- `note`
- `warning`
- `danger`
- `tip`
- `info`
- `success`
- `important`
- `caution`
- `example`
- `faq`
- `todo`

Rules:
- Opening form is `:::<kind>` (or command-arg equivalent from parser pipeline).
- Unknown kind falls back to `note`.
- Content supports nested block parsing:
  - headings
  - lists
  - code blocks
  - markdown tables
  - paragraphs

Serialization behavior:
- Serializer emits `:::<kind>` / content / `:::`.

## 3. Inline Syntax

Inline parsing is supported in paragraphs, headings, list items, and table cells.

### 3.1 Emphasis/format marks

Supported forms:
- Bold: `**text**`
- Italic: `*text*`
- Strikethrough: `~~text~~`
- Underline: `_text_`
- Highlight: `<mark>text</mark>`
- Subscript: `<sub>text</sub>`
- Superscript: `<sup>text</sup>`
- Inline code: `` `text` ``

### 3.2 Links and media

Supported forms:
- Wiki link: `[[Page Title]]`
- Markdown link: `[label](url)`
- Command link: `:::link|label|url`
- Image: `![alt](src)`

### 3.3 Other inline tokens

Supported forms:
- Hard break token: `<br>` or `<br/>`
- Citation token (Deep Research style): `cite<ref>`

Citation behavior:
- Parsed as inline citation atom with `ref` attribute.
- Serialized back to the same token form.
- No URL resolution is performed by ZealotScript itself.
- Multiple adjacent citations are supported (e.g., `citeaciteb`).

## 4. Parsing Order and Precedence

Top-level parser precedence:
1. Command blocks (`:::`)
2. Single-line block parsers (headings)
3. Multi-line block parsers (blockquote, markdown table, list, code block)
4. Paragraph fallback

Inline parser precedence (high-level):
1. Escape handling (`\x`)
2. Inline atoms (images, links, wiki links, command links, hard break, citation)
3. Paired tag marks (`<mark>`, `<sub>`, `<sup>`, `<u>`)
4. Inline code backticks
5. Symmetric marks (`**`, `~~`, `*`, `_`)
6. Plain text fallback

## 5. Serialization Rules

- Blocks are separated with blank lines by default.
- Empty blocks use single newline spacing behavior.
- Inline marks are emitted in ZealotScript syntax (not HTML except tag-based marks).
- Plain unmarked text is emitted unescaped.
- Blockquotes emit `>` prefixes.
- Tables emit markdown or command form depending on header detection.

## 6. Editor Input Rules (Typing Conversions)

While typing in editor, these patterns auto-convert to marks/nodes:
- `**...**`, `*...*`, `~~...~~`, `` `...` ``, `_..._`
- `<mark>...</mark>`, `<sub>...</sub>`, `<sup>...</sup>`
- `[[...]]`
- `[...](...)` (on trailing whitespace)
- `:::link|label|url`

Note: Input rules affect typing-time conversion only. Parsing on load is handled by parser rules.

## 7. Editor Key Behavior

- `Home` moves caret to start of current text block line.
- `End` moves caret to end of current text block line.
- `Shift+Home` / `Shift+End` extend selection to those boundaries.
- This is enforced in editor key handling to normalize macOS behavior with Linux/Windows expectations.

## 8. Visual Conventions (Current Theme)

Admonitions are styled by `data-admonition` and include kind labels and icons.

Current icon mapping:
- `note` -> `notes.svg`
- `info` -> `data.svg`
- `tip` -> `bolt.svg`
- `warning` -> `priority.svg`
- `danger` -> `broken.svg`
- `success` -> `clean.svg`
- `important` -> `pin.svg`
- `caution` -> `rules.svg`
- `example` -> `science.svg`
- `faq` -> `speech.svg`
- `todo` -> `add_note.svg`

These are presentational defaults from SCSS and are not part of the wire syntax.

## 9. Known Limitations

- Fenced code block language tags are not preserved.
- Citation tokens are preserved but not resolved to URLs.
- Serializer may normalize some source formatting (e.g., table layout style).
- Escaping semantics are pragmatic and implementation-driven, not full CommonMark compliance.

## 10. Conformance Source

This spec reflects behavior in:
- `client/src/features/zealotscript/parser.ts`
- `client/src/features/zealotscript/parse/*.ts`
- `client/src/features/zealotscript/commands/*.ts`
- `client/src/features/zealotscript/serializer.ts`
- `client/src/features/zealotscript/zealotscript_editor.ts`
- `client/src/features/zealotscript/schema.ts`
