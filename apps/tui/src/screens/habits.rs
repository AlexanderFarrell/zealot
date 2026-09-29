//! Habits: a scrollable date grid of repeat statuses; cycle a cell's status.

use chrono::{Datelike, Duration, Local, NaiveDate};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use zealot_domain::repeat::RepeatEntryDto;

use super::{Cx, Pane, Screen, status_span};
use crate::app::HABIT_STATUSES;
use crate::msg::Load;
use crate::net::Net;

pub struct HabitsState {
    pub end: NaiveDate,
    pub days: i64,
    pub entries: Load<Vec<RepeatEntryDto>>,
    pub row: usize,
    pub col: usize,
    pub generation: u64,
}

impl HabitsState {
    pub fn new(today: NaiveDate) -> Self {
        Self {
            end: today,
            days: 14,
            entries: Load::Idle,
            row: 0,
            col: 13,
            generation: 0,
        }
    }

    /// Distinct habit titles in the loaded range, in first-seen order.
    pub fn rows(&self) -> Vec<(String, i64)> {
        let mut rows: Vec<(String, i64)> = Vec::new();
        if let Some(entries) = self.entries.loaded() {
            for entry in entries {
                if !rows.iter().any(|(t, _)| *t == entry.item.title) {
                    rows.push((entry.item.title.clone(), entry.item.item_id));
                }
            }
        }
        rows
    }

    /// The calendar date shown in grid column `col`.
    pub fn grid_date(&self, col: usize) -> NaiveDate {
        self.end - Duration::days(self.days - 1 - col as i64)
    }

    /// Advance the selected cell's status, optimistically firing the update.
    fn cycle(&self, net: &Net) {
        let rows = self.rows();
        let Some((_, item_id)) = rows.get(self.row).cloned() else {
            return;
        };
        let date = self.grid_date(self.col);
        let date_str = date.format("%Y-%m-%d").to_string();
        let current = self
            .entries
            .loaded()
            .and_then(|entries| {
                entries
                    .iter()
                    .find(|e| e.item.item_id == item_id && e.date == date_str)
            })
            .map(|e| e.status.clone())
            .unwrap_or_else(|| "Not Complete".to_string());
        let idx = HABIT_STATUSES
            .iter()
            .position(|s| *s == current)
            .unwrap_or(0);
        let next = HABIT_STATUSES[(idx + 1) % HABIT_STATUSES.len()];
        net.set_habit_status(item_id, date, next);
    }
}

impl Pane for HabitsState {
    fn load(&mut self, net: &Net, gen_id: u64) {
        self.generation = gen_id;
        self.entries = Load::Loading;
        let start = self.end - Duration::days(self.days - 1);
        net.load_habits_range(start, self.end, gen_id);
    }

    fn on_key(&mut self, key: KeyEvent, cx: &mut Cx) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => cx.back(),
            KeyCode::Char('j') | KeyCode::Down => {
                let rows = self.rows().len();
                if rows > 0 {
                    self.row = (self.row + 1).min(rows - 1);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => self.row = self.row.saturating_sub(1),
            KeyCode::Char('h') | KeyCode::Left => self.col = self.col.saturating_sub(1),
            KeyCode::Char('l') | KeyCode::Right => {
                self.col = (self.col + 1).min(self.days as usize - 1);
            }
            KeyCode::Char('[') => {
                self.end -= Duration::days(7);
                cx.reload();
            }
            KeyCode::Char(']') => {
                self.end += Duration::days(7);
                cx.reload();
            }
            KeyCode::Char('t') => {
                self.end = Local::now().date_naive();
                cx.reload();
            }
            KeyCode::Char(' ') => self.cycle(cx.net),
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let start = self.grid_date(0);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Screen::Habits.titled(&format!(
                "Habits {} → {} (Space cycle · [ ] shift week)",
                start.format("%b %d"),
                self.end.format("%b %d")
            )))
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let entries = match &self.entries {
            Load::Loaded(entries) => entries,
            Load::Error(e) => {
                frame.render_widget(
                    Paragraph::new(format!("Error: {e}")).style(Style::default().fg(Color::Red)),
                    inner,
                );
                return;
            }
            _ => {
                frame.render_widget(
                    Paragraph::new("Loading…").style(Style::default().fg(Color::DarkGray)),
                    inner,
                );
                return;
            }
        };

        let rows = self.rows();
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
        for col in 0..self.days as usize {
            let date = self.grid_date(col);
            let style = if matches!(date.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) {
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
                if row_idx == self.row {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            )];
            for col in 0..self.days as usize {
                let date_str = self.grid_date(col).format("%Y-%m-%d").to_string();
                let glyph = entries
                    .iter()
                    .find(|e| e.item.item_id == *item_id && e.date == date_str)
                    .map(|e| status_span(&e.status))
                    .unwrap_or_else(|| Span::styled("·", Style::default().fg(Color::DarkGray)));
                let selected = row_idx == self.row && col == self.col;
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
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max - 1).collect();
        format!("{cut}…")
    }
}
