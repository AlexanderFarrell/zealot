use chrono::Local;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use super::{display_title, format_clock, status_span};
use crate::app::App;
use crate::msg::Load;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let state = &app.today;
    let title = if state.date == Local::now().date_naive() {
        format!(" {} (today) ", state.date.format("%A %Y-%m-%d"))
    } else {
        format!(" {} ", state.date.format("%A %Y-%m-%d"))
    };

    let outer = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    match &state.data {
        Load::Loaded(data) => {
            let [left, right] =
                Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
                    .areas(inner);
            let [plan_area, journal_area] =
                Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)])
                    .areas(left);
            let [habits_area, blocks_area] =
                Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)])
                    .areas(right);

            // Plan.
            let plan_items: Vec<ListItem> = if data.plan.is_empty() {
                vec![ListItem::new(Span::styled(
                    "nothing planned",
                    Style::default().fg(Color::DarkGray),
                ))]
            } else {
                data.plan
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
                    .collect()
            };
            frame.render_widget(
                List::new(plan_items).block(section(" Plan ")),
                plan_area,
            );

            // Journal (day comments) + input line.
            let mut journal_lines: Vec<Line> = data
                .comments
                .iter()
                .map(|comment| {
                    let time = comment.timestamp.get(11..16).unwrap_or("").to_string();
                    Line::from(vec![
                        Span::styled(time, Style::default().fg(Color::DarkGray)),
                        Span::raw(" "),
                        Span::styled(
                            display_title(&comment.item),
                            Style::default().fg(Color::Magenta),
                        ),
                        Span::raw(" "),
                        Span::raw(comment.content.replace('\n', " ")),
                    ])
                })
                .collect();
            if let Some(input) = &state.input {
                journal_lines.push(Line::from(vec![
                    Span::styled("✎ ", Style::default().fg(Color::Yellow)),
                    Span::raw(input.clone()),
                    Span::styled("▏", Style::default().fg(Color::Yellow)),
                ]));
            } else if data.comments.is_empty() {
                journal_lines.push(Line::from(Span::styled(
                    "press i to journal",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            frame.render_widget(
                Paragraph::new(journal_lines).block(section(" Journal ")),
                journal_area,
            );

            // Habits (selectable).
            let habit_items: Vec<ListItem> = data
                .habits
                .iter()
                .map(|entry| {
                    let mut spans = vec![
                        status_span(&entry.status),
                        Span::raw(" "),
                        Span::raw(display_title(&entry.item)),
                    ];
                    if !entry.comment.is_empty() {
                        spans.push(Span::styled(
                            format!("  — {}", entry.comment),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                    ListItem::new(Line::from(spans))
                })
                .collect();
            let done = data
                .habits
                .iter()
                .filter(|e| e.status == "Complete")
                .count();
            let mut habit_state =
                ListState::default().with_selected(Some(state.selected_habit));
            frame.render_stateful_widget(
                List::new(habit_items)
                    .block(section(&format!(
                        " Habits {done}/{} (Space to cycle) ",
                        data.habits.len()
                    )))
                    .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
                habits_area,
                &mut habit_state,
            );

            // Time blocks.
            let block_lines: Vec<Line> = if data.blocks.is_empty() {
                vec![Line::from(Span::styled(
                    "no time blocks",
                    Style::default().fg(Color::DarkGray),
                ))]
            } else {
                data.blocks
                    .iter()
                    .map(|block| {
                        let mut spans = vec![
                            Span::styled(
                                format!(
                                    "{:>5}–{:<5} ",
                                    format_clock(block.start_min),
                                    format_clock(block.end_min)
                                ),
                                Style::default().fg(Color::Yellow),
                            ),
                            Span::raw(display_title(&block.item)),
                        ];
                        if !block.note.is_empty() {
                            spans.push(Span::styled(
                                format!("  — {}", block.note),
                                Style::default().fg(Color::DarkGray),
                            ));
                        }
                        Line::from(spans)
                    })
                    .collect()
            };
            frame.render_widget(
                Paragraph::new(block_lines).block(section(" Time blocks ")),
                blocks_area,
            );
        }
        Load::Loading | Load::Idle => {
            frame.render_widget(
                Paragraph::new("Loading…").style(Style::default().fg(Color::DarkGray)),
                inner,
            );
        }
        Load::Error(e) => {
            frame.render_widget(
                Paragraph::new(format!("Error: {e}\n\nPress R to retry."))
                    .style(Style::default().fg(Color::Red)),
                inner,
            );
        }
    }
}

fn section(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .title(title.to_string())
        .border_style(Style::default().fg(Color::DarkGray))
}
