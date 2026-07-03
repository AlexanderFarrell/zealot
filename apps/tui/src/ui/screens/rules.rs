use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use crate::app::App;
use crate::msg::Load;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let [list_area, detail_area] =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).areas(area);

    let list_block = Block::default()
        .borders(Borders::ALL)
        .title(" Rules (r run, e edit script) ")
        .border_style(Style::default().fg(Color::Cyan));

    match &app.rules.rules {
        Load::Loaded(rules) => {
            let items: Vec<ListItem> = rules
                .iter()
                .map(|rule| {
                    let enabled = if rule.enabled {
                        Span::styled("●", Style::default().fg(Color::Green))
                    } else {
                        Span::styled("○", Style::default().fg(Color::DarkGray))
                    };
                    let error = if rule.last_error.as_deref().is_some_and(|e| !e.is_empty()) {
                        Span::styled(" ✘", Style::default().fg(Color::Red))
                    } else {
                        Span::raw("")
                    };
                    ListItem::new(Line::from(vec![
                        enabled,
                        Span::raw(" "),
                        Span::styled(
                            rule.name.clone(),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("  {}", rule.trigger.kind_str()),
                            Style::default().fg(Color::Yellow),
                        ),
                        error,
                    ]))
                })
                .collect();
            let mut state = ListState::default().with_selected(if rules.is_empty() {
                None
            } else {
                Some(app.rules.selected)
            });
            frame.render_stateful_widget(
                List::new(items)
                    .block(list_block)
                    .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
                list_area,
                &mut state,
            );

            // Detail pane.
            let detail_block = Block::default()
                .borders(Borders::ALL)
                .title(" Detail ")
                .border_style(Style::default().fg(Color::DarkGray));
            if let Some(rule) = rules.get(app.rules.selected) {
                let mut lines = vec![
                    Line::from(vec![
                        Span::styled(
                            rule.name.clone(),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("  #{}", rule.rule_id),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                    Line::from(Span::raw(rule.description.clone())),
                    Line::from(vec![
                        Span::styled("Trigger: ", Style::default().fg(Color::Cyan)),
                        Span::raw(
                            serde_json::to_string(&rule.trigger).unwrap_or_default(),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("Last run: ", Style::default().fg(Color::Cyan)),
                        Span::raw(
                            rule.last_run_at
                                .map(|at| at.format("%Y-%m-%d %H:%M:%S").to_string())
                                .unwrap_or_else(|| "never".into()),
                        ),
                    ]),
                    Line::default(),
                ];
                if let Some(output) = rule.last_output.as_deref().filter(|o| !o.is_empty()) {
                    lines.push(Line::from(Span::styled(
                        "Last output:",
                        Style::default().fg(Color::Green),
                    )));
                    for line in output.lines().take(20) {
                        lines.push(Line::from(Span::raw(line.to_string())));
                    }
                }
                if let Some(error) = rule.last_error.as_deref().filter(|e| !e.is_empty()) {
                    lines.push(Line::from(Span::styled(
                        "Last error:",
                        Style::default().fg(Color::Red),
                    )));
                    for line in error.lines().take(20) {
                        lines.push(Line::from(Span::styled(
                            line.to_string(),
                            Style::default().fg(Color::Red),
                        )));
                    }
                }
                frame.render_widget(
                    Paragraph::new(lines)
                        .block(detail_block)
                        .wrap(Wrap { trim: false }),
                    detail_area,
                );
            } else {
                frame.render_widget(
                    Paragraph::new("No rules.").block(detail_block),
                    detail_area,
                );
            }
        }
        Load::Error(e) => {
            frame.render_widget(
                Paragraph::new(format!("Error: {e}"))
                    .style(Style::default().fg(Color::Red))
                    .block(list_block),
                area,
            );
        }
        _ => {
            frame.render_widget(
                Paragraph::new("Loading…")
                    .style(Style::default().fg(Color::DarkGray))
                    .block(list_block),
                area,
            );
        }
    }
}
