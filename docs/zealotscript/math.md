# ZealotScript — Math (LaTeX)

Math expressions are rendered via **KaTeX**. Both inline and display-mode (block) math are
supported.

## Inline math

Wrap a LaTeX expression in single dollar signs with no leading or trailing space:

```
The energy equation is $E = mc^2$ where $c$ is the speed of light.
```

Renders the expression inline within the surrounding paragraph text.

**Rules:**
- No space between `$` and the first/last character of the expression: `$E = mc^2$` ✓,
  `$ E $` ✗ (treated as literal text)
- Nesting is not supported — the first `$` after the opening `$` closes the expression

## Block math

Use the `:::math` fenced block for display-mode (centered, larger) expressions:

```
:::math
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
:::
```

The block is rendered centered with display-mode KaTeX output.

**Multi-line expressions** are supported — the entire content between `:::math` and `:::` is
passed as a single LaTeX string:

```
:::math
\begin{pmatrix}
  a & b \\
  c & d
\end{pmatrix}
\begin{pmatrix}
  x \\
  y
\end{pmatrix}
=
\begin{pmatrix}
  ax + by \\
  cx + dy
\end{pmatrix}
:::
```

## Error handling

If a LaTeX expression is invalid, the raw source is displayed in red instead of crashing.
This makes it easy to spot typos without losing content.

## Common LaTeX quick-reference

| What you want | LaTeX |
|---|---|
| Fraction | `\frac{a}{b}` |
| Square root | `\sqrt{x}` or `\sqrt[n]{x}` |
| Superscript | `x^{2}` |
| Subscript | `x_{i}` |
| Sum | `\sum_{i=0}^{n}` |
| Integral | `\int_{a}^{b} f(x)\,dx` |
| Greek letters | `\alpha`, `\beta`, `\gamma`, `\pi`, `\Sigma`, … |
| Infinity | `\infty` |
| Approximately | `\approx` |
| Not equal | `\neq` |
| Less/greater | `\leq`, `\geq` |
| Absolute value | `|x|` or `\lvert x \rvert` |
| Matrix | `\begin{pmatrix} … \end{pmatrix}` |
| Aligned equations | `\begin{aligned} … \end{aligned}` |

For a full reference see the [KaTeX supported functions list](https://katex.org/docs/supported.html).

## Implementation notes

- **Library**: [KaTeX](https://katex.org/) (fast client-side rendering, no server required)
- **Inline node**: `math_inline` — inline atom with `src` attribute
- **Block node**: `math_block` — block atom with `src` attribute
- **Rendering**: done in `zealotscript_view.ts` after mount via `katex.renderToString()`
- **Editor**: shows raw LaTeX source (no live preview in edit mode)
- **Source files**:
  - `packages/ui/src/zealotscript/schema.ts` — `math_inline`, `math_block` node specs
  - `packages/ui/src/zealotscript/parse/parse_math_block.ts` — `:::math` block parser
  - `packages/ui/src/zealotscript/parse/parse_inline.ts` — `$…$` inline rule
  - `packages/ui/src/zealotscript/serializer.ts` — round-trip serialization
  - `packages/ui/src/zealotscript/zealotscript_view.ts` — KaTeX render pass
  - `packages/content/src/css/tags.scss` — `.zealot-math-block`, `.zealot-math-inline`, `.zealot-math-error`
