use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use zealot_domain::item::SearchScope;

use super::display_title;
use crate::app::App;
use crate::msg::Load;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let [input_area, results_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).areas(area);

    let scope = match app.search.scope {
        SearchScope::Title => "title",
        SearchScope::Content => "content",
        SearchScope::Heading => "heading",
    };
    let regex = if app.search.regex { " · regex" } else { "" };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("🔎 ", Style::default().fg(Color::Cyan)),
            Span::raw(&app.search.query),
            Span::styled("▏", Style::default().fg(Color::Cyan)),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Search — {scope}{regex} (Tab scope, Ctrl-R regex) "))
                .border_style(Style::default().fg(Color::Cyan)),
        ),
        input_area,
    );

    let results_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    match &app.search.results {
        Load::Loaded(results) => {
            let items: Vec<ListItem> = results
                .iter()
                .map(|result| {
                    let mut lines = vec![Line::from(vec![
                        Span::styled(
                            format!("#{} ", result.item.item_id),
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::styled(
                            display_title(&result.item),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!(
                                "  {}",
                                result
                                    .item
                                    .types
                                    .iter()
                                    .map(|t| t.name.clone())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ])];
                    if let Some(snippet) = &result.snippet {
                        lines.push(Line::from(Span::styled(
                            format!("   {}", snippet.replace('\n', " ")),
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                    ListItem::new(lines)
                })
                .collect();
            let mut state = ListState::default().with_selected(if results.is_empty() {
                None
            } else {
                Some(app.search.selected)
            });
            frame.render_stateful_widget(
                List::new(items)
                    .block(results_block.title(format!(" {} result(s) ", results.len())))
                    .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
                results_area,
                &mut state,
            );
        }
        Load::Loading => {
            frame.render_widget(
                Paragraph::new("Searching…")
                    .style(Style::default().fg(Color::DarkGray))
                    .block(results_block),
                results_area,
            );
        }
        Load::Idle => {
            frame.render_widget(
                Paragraph::new("Type to search your wiki.")
                    .style(Style::default().fg(Color::DarkGray))
                    .block(results_block),
                results_area,
            );
        }
        Load::Error(e) => {
            frame.render_widget(
                Paragraph::new(format!("Error: {e}"))
                    .style(Style::default().fg(Color::Red))
                    .block(results_block),
                results_area,
            );
        }
    }
}
