use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use super::display_title;
use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let [tree_area, preview_area] =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(area);

    let visible = app.visible_nodes();

    // Tree.
    let items: Vec<ListItem> = visible
        .iter()
        .map(|&idx| {
            let node = &app.browse.nodes[idx];
            let arrow = if node.expanded {
                "▾ "
            } else if node.children_loaded {
                "▸ "
            } else {
                "› "
            };
            ListItem::new(Line::from(vec![
                Span::raw("  ".repeat(node.depth)),
                Span::styled(arrow, Style::default().fg(Color::DarkGray)),
                Span::raw(display_title(&node.item)),
            ]))
        })
        .collect();

    let title = if app.browse.loading {
        " Items (loading…) "
    } else {
        " Items "
    };
    let mut state = ListState::default().with_selected(if visible.is_empty() {
        None
    } else {
        Some(app.browse.selected.min(visible.len() - 1))
    });
    frame.render_stateful_widget(
        List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
        tree_area,
        &mut state,
    );

    // Preview.
    let preview_block = Block::default()
        .borders(Borders::ALL)
        .title(" Preview (Enter to open, e to edit) ")
        .border_style(Style::default().fg(Color::DarkGray));
    if let Some(&idx) = visible.get(app.browse.selected.min(visible.len().saturating_sub(1))) {
        let node = &app.browse.nodes[idx];
        let rendered = zealot_zscript::render(&node.item.content);
        let text = ansi_to_tui::IntoText::into_text(&rendered)
            .unwrap_or_else(|_| node.item.content.clone().into());
        frame.render_widget(
            Paragraph::new(text)
                .block(preview_block)
                .wrap(Wrap { trim: false }),
            preview_area,
        );
    } else {
        frame.render_widget(
            Paragraph::new("No items.").block(preview_block),
            preview_area,
        );
    }
}
