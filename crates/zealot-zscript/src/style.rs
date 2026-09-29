//! ANSI styling helpers gated on a process-global color flag.

use std::sync::atomic::{AtomicBool, Ordering};

static COLOR_ENABLED: AtomicBool = AtomicBool::new(false);
static HYPERLINKS_ENABLED: AtomicBool = AtomicBool::new(true);
static LINK_TARGETS_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn set_color_enabled(enabled: bool) {
    COLOR_ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn color_enabled() -> bool {
    COLOR_ENABLED.load(Ordering::Relaxed)
}

/// When on, wikilinks are wrapped in OSC 8 hyperlinks carrying a
/// `zealot:<title>` target (like markdown links already carry their URL), so a
/// structured consumer such as the TUI can recover each link's destination.
/// Off by default: the CLI keeps rendering plain, non-navigable wikilinks.
pub fn set_link_targets_enabled(enabled: bool) {
    LINK_TARGETS_ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn link_targets_enabled() -> bool {
    LINK_TARGETS_ENABLED.load(Ordering::Relaxed)
}

/// OSC 8 hyperlinks are on by default with color; the TUI turns them off
/// because its ANSI-to-span parser only understands SGR sequences.
pub fn set_hyperlinks_enabled(enabled: bool) {
    HYPERLINKS_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Wrap `text` in an SGR sequence when color output is enabled.
pub fn paint(codes: &str, text: &str) -> String {
    if color_enabled() && !text.is_empty() {
        format!("\x1b[{codes}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

pub fn bold(text: &str) -> String {
    paint("1", text)
}

pub fn dim(text: &str) -> String {
    paint("2", text)
}

pub fn italic(text: &str) -> String {
    paint("3", text)
}

pub fn underline(text: &str) -> String {
    paint("4", text)
}

pub fn cyan(text: &str) -> String {
    paint("36", text)
}

pub fn green(text: &str) -> String {
    paint("32", text)
}

pub fn yellow(text: &str) -> String {
    paint("33", text)
}

pub fn red(text: &str) -> String {
    paint("31", text)
}

pub fn blue(text: &str) -> String {
    paint("34", text)
}

pub fn magenta(text: &str) -> String {
    paint("35", text)
}

/// Emit an OSC 8 hyperlink to `uri` with a plain `label`, regardless of the
/// `hyperlinks` flag. Used to carry a navigable target to structured consumers
/// (the TUI) without styling the label — they restyle it themselves.
pub fn link_target(uri: &str, label: &str) -> String {
    if color_enabled() {
        format!("\x1b]8;;{uri}\x1b\\{label}\x1b]8;;\x1b\\")
    } else {
        label.to_string()
    }
}

/// Terminal hyperlink (OSC 8) when enabled; styled label otherwise.
pub fn hyperlink(label: &str, url: &str) -> String {
    if color_enabled() && HYPERLINKS_ENABLED.load(Ordering::Relaxed) {
        format!("\x1b]8;;{url}\x1b\\{}\x1b]8;;\x1b\\", underline(label))
    } else if color_enabled() {
        underline(label)
    } else {
        label.to_string()
    }
}

/// Visible width of a string, ignoring ANSI escape sequences.
pub fn visible_width(s: &str) -> usize {
    let mut width = 0;
    let mut in_escape = false;
    for c in s.chars() {
        if in_escape {
            if c.is_ascii_alphabetic() || c == '\\' {
                in_escape = false;
            }
            continue;
        }
        if c == '\x1b' {
            in_escape = true;
            continue;
        }
        width += 1;
    }
    width
}
