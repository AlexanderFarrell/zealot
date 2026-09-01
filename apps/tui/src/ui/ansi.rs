use std::iter::Peekable;
use std::str::Chars;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};

/// Where a navigable link points once the user activates it.
#[derive(Debug, Clone)]
pub enum LinkTarget {
    /// A wiki item, opened in the viewer by title.
    Item(String),
    /// An external URL, handed to the system opener.
    Url(String),
}

/// A link discovered while parsing rendered content, tied to the span it
/// occupies so the viewer can highlight it and scroll it into view.
#[derive(Debug, Clone)]
pub struct BodyLink {
    pub line: usize,
    pub span: usize,
    pub target: LinkTarget,
}

/// Style applied to every link label. Magenta reads far brighter than blue on
/// dark terminals; underline keeps links legible when color is unavailable.
pub fn link_style() -> Style {
    Style::default()
        .fg(Color::Magenta)
        .add_modifier(Modifier::UNDERLINED)
}

/// Parse the renderer's ANSI (SGR + OSC 8 hyperlinks) into styled text, keep
/// the plain-text convenience wrapper for callers that don't need links.
pub fn sgr_to_text(input: &str) -> Text<'static> {
    render_links(input).0
}

/// Parse rendered ANSI into styled text plus the list of links it contains, in
/// document order. OSC 8 hyperlinks (emitted for wikilinks and markdown links)
/// carry the target; their labels are restyled with [`link_style`].
pub fn render_links(input: &str) -> (Text<'static>, Vec<BodyLink>) {
    let mut lines: Vec<Line> = Vec::new();
    let mut spans: Vec<Span> = Vec::new();
    let mut buf = String::new();
    let mut style = Style::default();
    let mut links: Vec<BodyLink> = Vec::new();
    // The open OSC 8 hyperlink, if any: its target and the label collected so far.
    let mut link: Option<(LinkTarget, String)> = None;

    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            // CSI (SGR) sequence.
            '\x1b' if chars.peek() == Some(&'[') => {
                chars.next();
                let mut code = String::new();
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() {
                        // Ignore styling toggles inside a link label — the label
                        // wears the uniform link style and the surrounding style
                        // (e.g. a heading's) is preserved across the link.
                        if c == 'm' && link.is_none() {
                            flush_span(&mut spans, &mut buf, style);
                            apply_sgr(&code, &mut style);
                        }
                        break;
                    }
                    code.push(c);
                }
            }
            // OSC sequence — we only care about OSC 8 hyperlinks.
            '\x1b' if chars.peek() == Some(&']') => {
                chars.next();
                let params = read_osc(&mut chars);
                if let Some(uri) = params.strip_prefix("8;;") {
                    if uri.is_empty() {
                        // Closing delimiter: emit the collected link label.
                        if let Some((target, label)) = link.take() {
                            push_link(&mut spans, &mut links, lines.len(), label, target);
                        }
                    } else {
                        // Opening delimiter: flush pending text, start collecting.
                        flush_span(&mut spans, &mut buf, style);
                        link = Some((parse_target(uri), String::new()));
                    }
                }
            }
            '\n' => {
                if let Some((target, label)) = link.take() {
                    push_link(&mut spans, &mut links, lines.len(), label, target);
                }
                flush_span(&mut spans, &mut buf, style);
                lines.push(Line::from(std::mem::take(&mut spans)));
            }
            _ => match &mut link {
                Some((_, label)) => label.push(ch),
                None => buf.push(ch),
            },
        }
    }

    if let Some((target, label)) = link.take() {
        push_link(&mut spans, &mut links, lines.len(), label, target);
    }
    flush_span(&mut spans, &mut buf, style);
    lines.push(Line::from(spans));
    (Text::from(lines), links)
}

fn push_link(
    spans: &mut Vec<Span<'static>>,
    links: &mut Vec<BodyLink>,
    line: usize,
    label: String,
    target: LinkTarget,
) {
    if label.is_empty() {
        return;
    }
    links.push(BodyLink {
        line,
        span: spans.len(),
        target,
    });
    spans.push(Span::styled(label, link_style()));
}

fn parse_target(uri: &str) -> LinkTarget {
    match uri.strip_prefix("zealot:") {
        Some(title) => LinkTarget::Item(title.to_string()),
        None => LinkTarget::Url(uri.to_string()),
    }
}

/// Read an OSC sequence's parameters up to its terminator (ST `ESC \` or BEL).
fn read_osc(chars: &mut Peekable<Chars>) -> String {
    let mut out = String::new();
    while let Some(c) = chars.next() {
        match c {
            '\x07' => break,
            '\x1b' => {
                if chars.peek() == Some(&'\\') {
                    chars.next();
                }
                break;
            }
            _ => out.push(c),
        }
    }
    out
}

fn flush_span(spans: &mut Vec<Span<'static>>, buf: &mut String, style: Style) {
    if !buf.is_empty() {
        spans.push(Span::styled(std::mem::take(buf), style));
    }
}

fn apply_sgr(code: &str, style: &mut Style) {
    let mut params = code
        .split(';')
        .map(|part| part.parse::<u8>().unwrap_or(0))
        .peekable();

    if params.peek().is_none() {
        *style = Style::default();
        return;
    }

    while let Some(param) = params.next() {
        match param {
            0 => *style = Style::default(),
            1 => *style = style.add_modifier(Modifier::BOLD),
            2 => *style = style.add_modifier(Modifier::DIM),
            3 => *style = style.add_modifier(Modifier::ITALIC),
            4 => *style = style.add_modifier(Modifier::UNDERLINED),
            7 => *style = style.add_modifier(Modifier::REVERSED),
            9 => *style = style.add_modifier(Modifier::CROSSED_OUT),
            22 => *style = style.remove_modifier(Modifier::BOLD | Modifier::DIM),
            23 => *style = style.remove_modifier(Modifier::ITALIC),
            24 => *style = style.remove_modifier(Modifier::UNDERLINED),
            27 => *style = style.remove_modifier(Modifier::REVERSED),
            29 => *style = style.remove_modifier(Modifier::CROSSED_OUT),
            30..=37 => *style = style.fg(basic_color(param - 30, false)),
            39 => *style = style.fg(Color::Reset),
            40..=47 => *style = style.bg(basic_color(param - 40, false)),
            49 => *style = style.bg(Color::Reset),
            90..=97 => *style = style.fg(basic_color(param - 90, true)),
            100..=107 => *style = style.bg(basic_color(param - 100, true)),
            38 => {
                if params.next() == Some(5)
                    && let Some(idx) = params.next()
                {
                    *style = style.fg(Color::Indexed(idx));
                }
            }
            48 => {
                if params.next() == Some(5)
                    && let Some(idx) = params.next()
                {
                    *style = style.bg(Color::Indexed(idx));
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Render ZealotScript exactly as the running TUI does, then parse it back.
    fn rendered(content: &str) -> (Text<'static>, Vec<BodyLink>) {
        zealot_zscript::set_color_enabled(true);
        zealot_zscript::style::set_hyperlinks_enabled(true);
        zealot_zscript::style::set_link_targets_enabled(true);
        render_links(&zealot_zscript::render(content))
    }

    #[test]
    fn wikilink_becomes_navigable_item() {
        let (text, links) = rendered("see [[Some Note]] here");
        assert_eq!(links.len(), 1);
        match &links[0].target {
            LinkTarget::Item(title) => assert_eq!(title, "Some Note"),
            other => panic!("expected item target, got {other:?}"),
        }
        // Label is styled magenta and the surrounding text survives intact.
        let span = &text.lines[links[0].line].spans[links[0].span];
        assert_eq!(span.content, "Some Note");
        assert_eq!(span.style.fg, Some(Color::Magenta));
        let joined: String = text.lines[0]
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert_eq!(joined, "see Some Note here");
    }

    #[test]
    fn markdown_link_keeps_external_url() {
        let (_, links) = rendered("read [docs](https://example.com/x) now");
        assert_eq!(links.len(), 1);
        match &links[0].target {
            LinkTarget::Url(url) => assert_eq!(url, "https://example.com/x"),
            other => panic!("expected url target, got {other:?}"),
        }
    }

    #[test]
    fn links_recovered_in_document_order() {
        let (_, links) = rendered("[[First]] then [second](https://e.com) then [[Third]]");
        let targets: Vec<String> = links
            .iter()
            .map(|l| match &l.target {
                LinkTarget::Item(t) => t.clone(),
                LinkTarget::Url(u) => u.clone(),
            })
            .collect();
        assert_eq!(targets, ["First", "https://e.com", "Third"]);
    }

    #[test]
    fn type_wikilink_uses_label() {
        let (text, links) = rendered("a [[type:Recipe]] link");
        assert_eq!(links.len(), 1);
        assert_eq!(
            text.lines[links[0].line].spans[links[0].span].content,
            "Recipe"
        );
    }

    #[test]
    fn plain_sgr_still_parses_without_links() {
        let (text, links) = rendered("**bold** text");
        assert!(links.is_empty());
        let joined: String = text.lines[0]
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert_eq!(joined, "bold text");
    }
}

fn basic_color(idx: u8, bright: bool) -> Color {
    match (idx, bright) {
        (0, false) => Color::Black,
        (1, false) => Color::Red,
        (2, false) => Color::Green,
        (3, false) => Color::Yellow,
        (4, false) => Color::Blue,
        (5, false) => Color::Magenta,
        (6, false) => Color::Cyan,
        (7, false) => Color::Gray,
        (0, true) => Color::DarkGray,
        (1, true) => Color::LightRed,
        (2, true) => Color::LightGreen,
        (3, true) => Color::LightYellow,
        (4, true) => Color::LightBlue,
        (5, true) => Color::LightMagenta,
        (6, true) => Color::LightCyan,
        _ => Color::White,
    }
}
