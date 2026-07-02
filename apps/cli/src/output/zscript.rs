//! ZealotScript → ANSI terminal renderer.
//!
//! Hand-rolled line-based renderer for the ZealotScript dialect (see
//! docs/zealotscript/README.md): markdown plus `:::kind` fenced blocks,
//! `[[wikilinks]]`, `_underline_`, `<mark>`, and `$math$`. Emoji shortcodes
//! are already resolved to unicode at storage time, so they pass through.

use super::{blue, bold, cyan, dim, hyperlink, italic, magenta, paint, underline};

pub fn render(content: &str) -> String {
    let mut out = Vec::new();
    let mut lines = content.lines().peekable();
    let mut in_code_fence = false;
    let mut admonition: Option<String> = None;

    while let Some(line) = lines.next() {
        let trimmed = line.trim_end();

        // Code fences pass through dimmed, verbatim.
        if trimmed.trim_start().starts_with("```") {
            in_code_fence = !in_code_fence;
            let lang = trimmed.trim_start().trim_start_matches('`');
            if in_code_fence && !lang.is_empty() {
                out.push(dim(&format!("┌─ {lang}")));
            } else {
                out.push(dim(if in_code_fence { "┌─" } else { "└─" }));
            }
            continue;
        }
        if in_code_fence {
            out.push(format!("{} {}", dim("│"), paint("38;5;250", trimmed)));
            continue;
        }

        // ::: fenced blocks — admonitions, math, youtube.
        if let Some(rest) = trimmed.trim_start().strip_prefix(":::") {
            let rest = rest.trim();
            if rest.is_empty() {
                admonition = None;
                out.push(String::new());
                continue;
            }
            let (kind, arg) = match rest.split_once(char::is_whitespace) {
                Some((k, a)) => (k.to_lowercase(), a.trim().to_string()),
                None => (rest.to_lowercase(), String::new()),
            };
            match kind.as_str() {
                "youtube" => {
                    let url = format!("https://youtu.be/{arg}");
                    out.push(format!("  ▶ {}", hyperlink(&url, &url)));
                    admonition = None;
                }
                "math" => {
                    admonition = Some("math".to_string());
                    out.push(dim("  ∑ math"));
                }
                _ => {
                    let label = if arg.is_empty() {
                        kind.to_uppercase()
                    } else {
                        format!("{} — {}", kind.to_uppercase(), arg)
                    };
                    out.push(format!(
                        "{} {}",
                        admonition_gutter(&kind),
                        bold(&admonition_color(&kind, &label))
                    ));
                    admonition = Some(kind);
                }
            }
            continue;
        }

        let (gutter, body) = match admonition.as_deref() {
            Some("math") => (format!("{}   ", dim("  ")), trimmed.to_string()),
            Some(kind) => (format!("{} ", admonition_gutter(kind)), trimmed.to_string()),
            None => (String::new(), trimmed.to_string()),
        };

        // Tables: gather a run of |-rows and align them.
        if body.trim_start().starts_with('|') {
            let mut rows = vec![body.clone()];
            while lines
                .peek()
                .is_some_and(|l| l.trim_start().starts_with('|'))
            {
                rows.push(lines.next().unwrap().trim_end().to_string());
            }
            for rendered in render_table(&rows) {
                out.push(format!("{gutter}{rendered}"));
            }
            continue;
        }

        out.push(format!("{gutter}{}", render_line(&body)));
    }

    out.join("\n")
}

fn render_line(line: &str) -> String {
    // Headings.
    let hashes = line.chars().take_while(|&c| c == '#').count();
    if (1..=6).contains(&hashes) && line[hashes..].starts_with(' ') {
        let text = render_inline(line[hashes..].trim());
        return match hashes {
            1 => bold(&paint("4;36", &text.to_uppercase())),
            2 => bold(&cyan(&text)),
            3 => bold(&text),
            _ => bold(&dim(&text)),
        };
    }

    // Blockquote.
    if let Some(rest) = line.trim_start().strip_prefix("> ") {
        return format!("{} {}", dim("┃"), italic(&render_inline(rest)));
    }
    if line.trim_start() == ">" {
        return dim("┃").to_string();
    }

    // Lists (preserve indentation for nesting).
    let indent_len = line.len() - line.trim_start().len();
    let (indent, rest) = line.split_at(indent_len);
    if let Some(item) = rest.strip_prefix("- [x] ").or(rest.strip_prefix("- [X] ")) {
        return format!("{indent}{} {}", paint("32", "☑"), dim(&render_inline(item)));
    }
    if let Some(item) = rest.strip_prefix("- [ ] ") {
        return format!("{indent}☐ {}", render_inline(item));
    }
    if let Some(item) = rest.strip_prefix("- ").or(rest.strip_prefix("* ")) {
        return format!("{indent}{} {}", cyan("•"), render_inline(item));
    }
    if let Some((num, item)) = split_ordered_marker(rest) {
        return format!("{indent}{} {}", cyan(&format!("{num}.")), render_inline(item));
    }

    // Horizontal rule.
    if !line.is_empty() && line.chars().all(|c| c == '-' || c == '_' || c == '*') && line.len() >= 3
    {
        return dim(&"─".repeat(40));
    }

    render_inline(line)
}

fn split_ordered_marker(s: &str) -> Option<(&str, &str)> {
    let digits = s.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 {
        return None;
    }
    let rest = &s[digits..];
    rest.strip_prefix(". ").map(|item| (&s[..digits], item))
}

/// Inline markup: **bold**, *italic*, ~~strike~~, _underline_, `code`,
/// [[wikilinks]], [links](url), <mark>, $math$.
fn render_inline(text: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Escaped character.
        if c == '\\' && i + 1 < chars.len() {
            out.push(chars[i + 1]);
            i += 2;
            continue;
        }

        // Wikilink [[Item]] / [[type:Type]].
        if c == '[' && chars.get(i + 1) == Some(&'[') {
            if let Some(end) = find_seq(&chars, i + 2, &[']', ']']) {
                let inner: String = chars[i + 2..end].iter().collect();
                let label = inner.split_once(':').map(|(_, l)| l).unwrap_or(&inner);
                out.push_str(&blue(&underline(label)));
                i = end + 2;
                continue;
            }
        }

        // Markdown link [label](url).
        if c == '[' {
            if let Some(close) = find_char(&chars, i + 1, ']') {
                if chars.get(close + 1) == Some(&'(') {
                    if let Some(paren) = find_char(&chars, close + 2, ')') {
                        let label: String = chars[i + 1..close].iter().collect();
                        let url: String = chars[close + 2..paren].iter().collect();
                        out.push_str(&hyperlink(&blue(&label), &url));
                        i = paren + 1;
                        continue;
                    }
                }
            }
        }

        // Inline code.
        if c == '`' {
            if let Some(end) = find_char(&chars, i + 1, '`') {
                let code: String = chars[i + 1..end].iter().collect();
                out.push_str(&paint("36;48;5;236", &code));
                i = end + 1;
                continue;
            }
        }

        // Bold / italic.
        if c == '*' {
            if chars.get(i + 1) == Some(&'*') {
                if let Some(end) = find_seq(&chars, i + 2, &['*', '*']) {
                    let inner: String = chars[i + 2..end].iter().collect();
                    out.push_str(&bold(&render_inline(&inner)));
                    i = end + 2;
                    continue;
                }
            } else if let Some(end) = find_char(&chars, i + 1, '*') {
                let inner: String = chars[i + 1..end].iter().collect();
                out.push_str(&italic(&render_inline(&inner)));
                i = end + 1;
                continue;
            }
        }

        // Strikethrough.
        if c == '~' && chars.get(i + 1) == Some(&'~') {
            if let Some(end) = find_seq(&chars, i + 2, &['~', '~']) {
                let inner: String = chars[i + 2..end].iter().collect();
                out.push_str(&paint("9", &inner));
                i = end + 2;
                continue;
            }
        }

        // Underline (only when the underscores hug non-space text).
        if c == '_' && chars.get(i + 1).is_some_and(|n| !n.is_whitespace()) {
            if let Some(end) = find_char(&chars, i + 1, '_') {
                if chars[end - 1] != ' ' {
                    let inner: String = chars[i + 1..end].iter().collect();
                    out.push_str(&underline(&inner));
                    i = end + 1;
                    continue;
                }
            }
        }

        // Inline math ($…$, no space hugging the dollars).
        if c == '$' && chars.get(i + 1).is_some_and(|n| !n.is_whitespace()) {
            if let Some(end) = find_char(&chars, i + 1, '$') {
                if end > i + 1 && !chars[end - 1].is_whitespace() {
                    let inner: String = chars[i + 1..end].iter().collect();
                    out.push_str(&italic(&magenta(&inner)));
                    i = end + 1;
                    continue;
                }
            }
        }

        // <mark>, <sub>, <sup>, <br> tags.
        if c == '<' {
            if let Some((rendered, next)) = render_tag(&chars, i) {
                out.push_str(&rendered);
                i = next;
                continue;
            }
        }

        out.push(c);
        i += 1;
    }

    out
}

fn render_tag(chars: &[char], start: usize) -> Option<(String, usize)> {
    let rest: String = chars[start..].iter().collect();
    for (open, close, style) in [
        ("<mark>", "</mark>", "7"),   // inverse video
        ("<sub>", "</sub>", "2"),
        ("<sup>", "</sup>", "2"),
    ] {
        if rest.starts_with(open) {
            if let Some(end) = rest.find(close) {
                let inner = &rest[open.len()..end];
                let consumed = end + close.len();
                return Some((paint(style, inner), start + consumed));
            }
        }
    }
    if rest.starts_with("<br>") {
        return Some(("\n".to_string(), start + 4));
    }
    None
}

fn find_char(chars: &[char], from: usize, target: char) -> Option<usize> {
    (from..chars.len()).find(|&j| chars[j] == target)
}

fn find_seq(chars: &[char], from: usize, target: &[char]) -> Option<usize> {
    if chars.len() < target.len() {
        return None;
    }
    (from..=chars.len() - target.len()).find(|&j| &chars[j..j + target.len()] == target)
}

fn admonition_gutter(kind: &str) -> String {
    paint(admonition_sgr(kind), "▐")
}

fn admonition_color(kind: &str, text: &str) -> String {
    paint(admonition_sgr(kind), text)
}

fn admonition_sgr(kind: &str) -> &'static str {
    match kind {
        "warning" | "caution" => "33",
        "danger" => "31",
        "tip" | "success" => "32",
        "important" => "35",
        "example" | "faq" => "36",
        "todo" => "33;1",
        _ => "34", // note, info, and unknown kinds
    }
}

/// Align a run of markdown table rows into box-drawn columns.
fn render_table(rows: &[String]) -> Vec<String> {
    let parsed: Vec<Vec<String>> = rows
        .iter()
        .filter(|r| !is_separator_row(r))
        .map(|r| {
            r.trim()
                .trim_matches('|')
                .split('|')
                .map(|c| c.trim().to_string())
                .collect()
        })
        .collect();
    if parsed.is_empty() {
        return Vec::new();
    }

    let cols = parsed.iter().map(Vec::len).max().unwrap_or(0);
    let mut widths = vec![0usize; cols];
    for row in &parsed {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(super::visible_width(&render_inline(cell)));
        }
    }

    let mut out = Vec::new();
    for (idx, row) in parsed.iter().enumerate() {
        let mut line = String::new();
        for (i, width) in widths.iter().enumerate() {
            let cell = row.get(i).map(String::as_str).unwrap_or("");
            let rendered = if idx == 0 {
                bold(&render_inline(cell))
            } else {
                render_inline(cell)
            };
            let pad = width - super::visible_width(&rendered).min(*width);
            line.push_str(&rendered);
            line.push_str(&" ".repeat(pad));
            if i + 1 < cols {
                line.push_str(&dim(" │ "));
            }
        }
        out.push(line.trim_end().to_string());
        if idx == 0 {
            out.push(dim(
                &widths
                    .iter()
                    .map(|w| "─".repeat(*w))
                    .collect::<Vec<_>>()
                    .join("─┼─"),
            ));
        }
    }
    out
}

fn is_separator_row(row: &str) -> bool {
    let inner = row.trim().trim_matches('|');
    !inner.is_empty()
        && inner
            .chars()
            .all(|c| c == '-' || c == ':' || c == '|' || c.is_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Colors are disabled in tests (global default), so rendering is plain text.

    #[test]
    fn renders_headings_and_lists_plain() {
        let out = render("# Title\n- [x] done\n- [ ] open\n- plain");
        assert!(out.contains("TITLE"));
        assert!(out.contains("☑ done"));
        assert!(out.contains("☐ open"));
        assert!(out.contains("• plain"));
    }

    #[test]
    fn strips_inline_markers() {
        let out = render("**bold** and *ital* and `code` and [[type:Recipe]]");
        assert_eq!(out, "bold and ital and code and Recipe");
    }

    #[test]
    fn renders_links_as_labels() {
        let out = render("see [docs](https://example.com) now");
        assert_eq!(out, "see docs now");
    }

    #[test]
    fn admonition_block_renders_label_and_body() {
        let out = render(":::warning\nCareful!\n:::");
        assert!(out.contains("WARNING"));
        assert!(out.contains("Careful!"));
    }

    #[test]
    fn tables_align_columns() {
        let out = render("| A | Long |\n|---|---|\n| 1 | 2 |");
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("A"));
        assert!(lines[2].starts_with("1"));
    }

    #[test]
    fn code_fences_pass_through() {
        let out = render("```rust\nlet x = 1;\n```");
        assert!(out.contains("let x = 1;"));
        assert!(out.contains("rust"));
    }
}
