# Outline lists

An outline is a nested list whose items carry hierarchical dotted-decimal numbers: `1.`, `1.1`,
`1.1.1`. It is a ZealotScript feature; Markdown has no equivalent, and other Markdown tools show
it as an unrecognised fenced block with readable numbered text.

```text
:::outline
1. Goals
  1.1 [x] Learn the editor
  1.2 [ ] Ship the feature
    1.2.1 Write the parser
2. Risks
:::
```

## Numbers

- **Numbers are derived from position.** Insert, delete, indent or move an item and every number
  renumbers. The numbers you type are ignored when the block is read.
- **The saved text always carries the numbers**, so anything that reads the raw content (the MCP
  tools, `grep_items`, another editor) can cite an item as "requirement 1.2.1" and find that line.
  The top level is written `1.`, deeper levels `1.2`, `1.2.1`.
- Depth comes from indentation (two spaces or a tab per level).
- A typed number is dropped only when it is the first token of an item. To start an item's text
  with a number, such as `1.5 million`, the saved form is `1. 1.5 million`, which reads back
  correctly.
- If a tool edits the raw text and changes the structure, the displayed numbers are right the next
  time the item loads, and the saved text is corrected the next time the editor saves it.

## Task checkboxes

An item may start with `[ ]` or `[x]` after its number. Mixed items are fine, as in task lists.
Typing `[ ] ` at the start of an outline item in the editor turns it into a task.

## In the editor

The toolbar has `• List`, `1. List` and `1.1 Outline` buttons. With the cursor in a list, each
converts that whole list (and its nested lists) to its format, keeping item text, nesting and
checkboxes. Outside a list, the button starts one. Tab and Shift-Tab indent and outdent. Enter on
an empty item leaves one level, then the list.
