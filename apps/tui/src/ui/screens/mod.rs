pub mod browse;
pub mod habits;
pub mod rules;
pub mod search;
pub mod today;
pub mod viewer;

use ratatui::style::{Color, Style};
use ratatui::text::Span;
use zealot_domain::item::ItemDto;

/// Item title prefixed with its Icon attribute, like the other clients.
pub fn display_title(item: &ItemDto) -> String {
    if let Some(serde_json::Value::String(icon)) = item.attributes.get("Icon") {
        format!("{icon} {}", item.title)
    } else {
        item.title.clone()
    }
}

pub fn status_span(status: &str) -> Span<'static> {
    match status {
        "Complete" => Span::styled("✔", Style::default().fg(Color::Green)),
        "Skip" => Span::styled("↷", Style::default().fg(Color::Yellow)),
        "Alternate" => Span::styled("◆", Style::default().fg(Color::Cyan)),
        _ => Span::styled("·", Style::default().fg(Color::DarkGray)),
    }
}

pub fn format_clock(min: i32) -> String {
    format!("{}:{:02}", min / 60, min % 60)
}
