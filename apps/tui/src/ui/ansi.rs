use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};

pub fn sgr_to_text(input: &str) -> Text<'static> {
    let mut lines = Vec::new();
    let mut spans = Vec::new();
    let mut buf = String::new();
    let mut style = Style::default();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\x1b' if chars.peek() == Some(&'[') => {
                chars.next();
                let mut code = String::new();
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() {
                        if c == 'm' {
                            flush_span(&mut spans, &mut buf, style);
                            apply_sgr(&code, &mut style);
                        }
                        break;
                    }
                    code.push(c);
                }
            }
            '\n' => {
                flush_span(&mut spans, &mut buf, style);
                lines.push(Line::from(std::mem::take(&mut spans)));
            }
            _ => buf.push(ch),
        }
    }

    flush_span(&mut spans, &mut buf, style);
    lines.push(Line::from(spans));
    Text::from(lines)
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
                if params.next() == Some(5) {
                    if let Some(idx) = params.next() {
                        *style = style.fg(Color::Indexed(idx));
                    }
                }
            }
            48 => {
                if params.next() == Some(5) {
                    if let Some(idx) = params.next() {
                        *style = style.bg(Color::Indexed(idx));
                    }
                }
            }
            _ => {}
        }
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
