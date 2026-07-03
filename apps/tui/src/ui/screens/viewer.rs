use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};

use super::display_title;
use crate::app::App;
use crate::msg::{LinkKind, Load};
use crate::ui::{ansi::sgr_to_text, centered_rect};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let Some(item) = &app.viewer.item else {
        frame.render_widget(
            Paragraph::new("No item open. Find one via Browse (2) or Search (3).")
                .style(Style::default().fg(Color::DarkGray)),
            area,
        );
        return;
    };

    let types = item
        .types
        .iter()
        .map(|t| t.name.clone())
        .collect::<Vec<_>>()
        .join(", ");
    let title = format!(
        " #{} {}{} ",
        item.item_id,
        display_title(item),
        if types.is_empty() {
            String::new()
        } else {
            format!(" · {types}")
        }
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Attribute strip on top, content below.
    let attrs: Vec<String> = match &item.attributes {
        serde_json::Value::Object(map) => map
            .iter()
            .filter(|(k, _)| *k != "Icon")
            .map(|(k, v)| {
                let value = match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                format!("{k}: {value}")
            })
            .collect(),
        _ => Vec::new(),
    };

    let [attr_area, content_area] = Layout::vertical([
        Constraint::Length(if attrs.is_empty() { 0 } else { 1 }),
        Constraint::Min(1),
    ])
    .areas(inner);

    if !attrs.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                attrs.join("  ·  "),
                Style::default().fg(Color::Yellow),
            ))),
            attr_area,
        );
    }

    frame.render_widget(
        Paragraph::new(sgr_to_text(&app.viewer.rendered))
            .wrap(Wrap { trim: false })
            .scroll((app.viewer.scroll, 0)),
        content_area,
    );

    if let Some((kind, slot, selected)) = &app.viewer.links {
        draw_links_overlay(frame, *kind, slot, *selected);
    }
}

fn draw_links_overlay(
    frame: &mut Frame,
    kind: LinkKind,
    slot: &Load<Vec<zealot_domain::item::ItemDto>>,
    selected: usize,
) {
    let title = match kind {
        LinkKind::Backlinks => " Backlinks ",
        LinkKind::Related => " Related ",
        LinkKind::Children => " Children ",
    };
    let area = centered_rect(60, 18, frame.area());
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Cyan));

    match slot {
        Load::Loaded(items) if items.is_empty() => {
            frame.render_widget(
                Paragraph::new("None. (Esc to close)")
                    .style(Style::default().fg(Color::DarkGray))
                    .block(block),
                area,
            );
        }
        Load::Loaded(items) => {
            let list_items: Vec<ListItem> = items
                .iter()
                .map(|item| {
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("#{} ", item.item_id),
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::raw(display_title(item)),
                    ]))
                })
                .collect();
            let mut state = ListState::default().with_selected(Some(selected));
            frame.render_stateful_widget(
                List::new(list_items)
                    .block(block)
                    .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
                area,
                &mut state,
            );
        }
        Load::Error(e) => {
            frame.render_widget(
                Paragraph::new(format!("Error: {e}"))
                    .style(Style::default().fg(Color::Red))
                    .block(block),
                area,
            );
        }
        _ => {
            frame.render_widget(
                Paragraph::new("Loading…")
                    .style(Style::default().fg(Color::DarkGray))
                    .block(block),
                area,
            );
        }
    }
}
