use chrono::Datelike;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use super::status_span;
use crate::app::App;
use crate::msg::Load;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let state = &app.habits;
    let start = app.habit_grid_date(0);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " Habits {} → {} (Space cycle, [ ] shift week) ",
            start.format("%b %d"),
            state.end.format("%b %d")
        ))
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    match &state.entries {
        Load::Loaded(entries) => {
            let rows = app.habit_rows();
            if rows.is_empty() {
                frame.render_widget(
                    Paragraph::new("No habit entries in this range.")
                        .style(Style::default().fg(Color::DarkGray)),
                    inner,
                );
                return;
            }

            let title_width = rows
                .iter()
                .map(|(t, _)| t.chars().count())
                .max()
                .unwrap_or(0)
                .min(28);

            let mut lines: Vec<Line> = Vec::new();

            // Header row: day-of-month numbers, weekend dimmed.
            let mut header = vec![Span::raw(" ".repeat(title_width + 2))];
            for col in 0..state.days as usize {
                let date = app.habit_grid_date(col);
                let style = if matches!(
                    date.weekday(),
                    chrono::Weekday::Sat | chrono::Weekday::Sun
                ) {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::Cyan)
                };
                header.push(Span::styled(format!("{:>3}", date.day()), style));
            }
            lines.push(Line::from(header));

            for (row_idx, (title, item_id)) in rows.iter().enumerate() {
                let mut spans = vec![Span::styled(
                    format!("{:<width$}  ", truncate(title, 28), width = title_width),
                    if row_idx == state.row {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                )];
                for col in 0..state.days as usize {
                    let date_str = app.habit_grid_date(col).format("%Y-%m-%d").to_string();
                    let glyph = entries
                        .iter()
                        .find(|e| e.item.item_id == *item_id && e.date == date_str)
                        .map(|e| status_span(&e.status))
                        .unwrap_or_else(|| {
                            Span::styled("·", Style::default().fg(Color::DarkGray))
                        });
                    let selected = row_idx == state.row && col == state.col;
                    let cell = if selected {
                        Span::styled(
                            format!("[{}]", glyph.content),
                            glyph.style.add_modifier(Modifier::REVERSED),
                        )
                    } else {
                        Span::styled(format!(" {} ", glyph.content), glyph.style)
                    };
                    spans.push(cell);
                }
                lines.push(Line::from(spans));
            }

            lines.push(Line::from(Span::styled(
                "legend: ✔ done  ↷ skip  ◆ alternate  · none",
                Style::default().fg(Color::DarkGray),
            )));

            frame.render_widget(Paragraph::new(lines), inner);
        }
        Load::Error(e) => {
            frame.render_widget(
                Paragraph::new(format!("Error: {e}")).style(Style::default().fg(Color::Red)),
                inner,
            );
        }
        _ => {
            frame.render_widget(
                Paragraph::new("Loading…").style(Style::default().fg(Color::DarkGray)),
                inner,
            );
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max - 1).collect();
        format!("{cut}…")
    }
}
