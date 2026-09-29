//! Today: the day dashboard. Four focusable sections — Plan, Habits, Blocks,
//! Journal — each a navigable list whose selected item opens in the viewer.

use chrono::{Duration, Local, NaiveDate};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use super::{Cx, Pane, Select, border, display_title, format_clock, status_span};
use crate::app::HABIT_STATUSES;
use crate::msg::{DayData, Load, Msg};
use crate::net::Net;

/// Which section of the Today dashboard currently has the cursor.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Plan,
    Habits,
    Blocks,
    Journal,
}

impl Section {
    /// Tab order, laid out to roughly follow the on-screen columns.
    const ORDER: [Section; 4] = [
        Section::Plan,
        Section::Habits,
        Section::Blocks,
        Section::Journal,
    ];

    fn step(self, delta: isize) -> Section {
        let pos = Self::ORDER.iter().position(|s| *s == self).unwrap_or(0) as isize;
        let len = Self::ORDER.len() as isize;
        Self::ORDER[((pos + delta).rem_euclid(len)) as usize]
    }
}

pub struct TodayState {
    pub date: NaiveDate,
    pub data: Load<DayData>,
    pub focus: Section,
    pub plan: Select,
    pub habits: Select,
    pub blocks: Select,
    pub journal: Select,
    /// Journal composer line, present while `i` is active.
    pub input: Option<String>,
    /// Item (title or id) day-comments are logged against, from the profile.
    pub journal_item: Option<String>,
    pub generation: u64,
}

impl TodayState {
    pub fn new(date: NaiveDate, journal_item: Option<String>) -> Self {
        Self {
            date,
            data: Load::Idle,
            focus: Section::Plan,
            plan: Select::default(),
            habits: Select::default(),
            blocks: Select::default(),
            journal: Select::default(),
            input: None,
            journal_item,
            generation: 0,
        }
    }

    /// Keep every section's cursor in range after data reloads.
    pub fn clamp_cursors(&mut self) {
        if let Some(data) = self.data.loaded() {
            self.plan.clamp(data.plan.len());
            self.habits.clamp(data.habits.len());
            self.blocks.clamp(data.blocks.len());
            self.journal.clamp(data.comments.len());
        }
    }

    fn cursor_mut(&mut self) -> &mut Select {
        match self.focus {
            Section::Plan => &mut self.plan,
            Section::Habits => &mut self.habits,
            Section::Blocks => &mut self.blocks,
            Section::Journal => &mut self.journal,
        }
    }

    fn focused_len(&self, data: &DayData) -> usize {
        match self.focus {
            Section::Plan => data.plan.len(),
            Section::Habits => data.habits.len(),
            Section::Blocks => data.blocks.len(),
            Section::Journal => data.comments.len(),
        }
    }

    /// The item id under the cursor in the focused section, if any.
    fn selected_item(&self) -> Option<i64> {
        let data = self.data.loaded()?;
        match self.focus {
            Section::Plan => data.plan.get(self.plan.index).map(|i| i.item_id),
            Section::Habits => data.habits.get(self.habits.index).map(|e| e.item.item_id),
            Section::Blocks => data.blocks.get(self.blocks.index).map(|b| b.item.item_id),
            Section::Journal => data
                .comments
                .get(self.journal.index)
                .map(|c| c.item.item_id),
        }
    }

    fn cycle_habit(&mut self, net: &Net) {
        let date = self.date;
        let index = self.habits.index;
        let Some(data) = self.data.loaded_mut() else {
            return;
        };
        let Some(entry) = data.habits.get_mut(index) else {
            return;
        };
        let current = HABIT_STATUSES
            .iter()
            .position(|s| *s == entry.status)
            .unwrap_or(0);
        let next = HABIT_STATUSES[(current + 1) % HABIT_STATUSES.len()];
        entry.status = next.to_string(); // optimistic; the Done() reply refetches
        net.set_habit_status(entry.item.item_id, date, next);
    }

    /// Key handling while the journal composer (`i`) is open.
    pub fn journal_input_key(&mut self, key: KeyEvent, cx: &mut Cx) {
        let Some(input) = &mut self.input else {
            return;
        };
        match key.code {
            KeyCode::Esc => self.input = None,
            KeyCode::Enter => {
                let text = input.trim().to_string();
                self.input = None;
                if text.is_empty() {
                    return;
                }
                let Some(title) = self.journal_item.clone() else {
                    cx.toast(
                        "no journal_item configured — set it in your zealot config profile",
                        true,
                    );
                    return;
                };
                // Resolve title → id → comment in one task chain.
                let net = cx.net.clone();
                tokio::spawn(async move {
                    let item = if let Ok(id) = title.parse::<i64>() {
                        net.client.get_item(id).await
                    } else {
                        net.client.get_item_by_title(&title).await
                    };
                    match item {
                        Ok(item) => net.add_comment(item.item_id, text),
                        Err(e) => {
                            let _ = net
                                .tx
                                .send(Msg::Done(Err(format!("journal item '{title}': {e}"))));
                        }
                    }
                });
            }
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Char(c) => input.push(c),
            _ => {}
        }
    }
}

impl Pane for TodayState {
    fn load(&mut self, net: &Net, gen_id: u64) {
        self.generation = gen_id;
        self.data = Load::Loading;
        net.load_day(self.date, gen_id);
    }

    fn on_key(&mut self, key: KeyEvent, cx: &mut Cx) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => cx.back(),
            KeyCode::Char('[') => {
                self.date -= Duration::days(1);
                cx.reload();
            }
            KeyCode::Char(']') => {
                self.date += Duration::days(1);
                cx.reload();
            }
            KeyCode::Char('t') => {
                self.date = Local::now().date_naive();
                cx.reload();
            }
            KeyCode::Tab => self.focus = self.focus.step(1),
            KeyCode::BackTab => self.focus = self.focus.step(-1),
            KeyCode::Char('j') | KeyCode::Down => {
                let len = self.data.loaded().map(|d| self.focused_len(d)).unwrap_or(0);
                self.cursor_mut().down(len);
            }
            KeyCode::Char('k') | KeyCode::Up => self.cursor_mut().up(),
            KeyCode::Char(' ') if self.focus == Section::Habits => self.cycle_habit(cx.net),
            KeyCode::Char('i') => self.input = Some(String::new()),
            KeyCode::Enter => {
                if let Some(id) = self.selected_item() {
                    cx.open(id);
                }
            }
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let today = Local::now().date_naive();
        let suffix = if self.date == today { " (today)" } else { "" };
        let outer = Block::default()
            .borders(Borders::ALL)
            .title(format!(
                " [1] {}{suffix}  ·  Tab switches section ",
                self.date.format("%A %Y-%m-%d")
            ))
            .border_style(Style::default().fg(Color::Cyan));
        let inner = outer.inner(area);
        frame.render_widget(outer, area);

        let data = match &self.data {
            Load::Loaded(data) => data,
            Load::Error(e) => {
                frame.render_widget(
                    Paragraph::new(format!("Error: {e}\n\nPress R to retry."))
                        .style(Style::default().fg(Color::Red)),
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

        let [left, right] =
            Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
                .areas(inner);
        let [plan_area, journal_area] =
            Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)]).areas(left);
        let [habits_area, blocks_area] =
            Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)]).areas(right);

        // ── Plan ──
        let plan_items = list_or_hint(
            data.plan.iter().map(|item| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("#{} ", item.item_id),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw(display_title(item)),
                ]))
            }),
            "nothing planned",
        );
        self.section(
            frame,
            plan_area,
            " Plan ",
            Section::Plan,
            plan_items,
            self.plan,
        );

        // ── Habits ──
        let done = data
            .habits
            .iter()
            .filter(|e| e.status == "Complete")
            .count();
        let habit_items = list_or_hint(
            data.habits.iter().map(|entry| {
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
            }),
            "no habits",
        );
        // The Habits panel doubles as a hint that "4" opens the full Habits screen.
        let habits_title = format!(" [4] Habits {done}/{} (Space cycles) ", data.habits.len());
        self.section(
            frame,
            habits_area,
            &habits_title,
            Section::Habits,
            habit_items,
            self.habits,
        );

        // ── Time blocks ──
        let block_items = list_or_hint(
            data.blocks.iter().map(|block| {
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
                ListItem::new(Line::from(spans))
            }),
            "no time blocks",
        );
        self.section(
            frame,
            blocks_area,
            " Time blocks ",
            Section::Blocks,
            block_items,
            self.blocks,
        );

        // ── Journal (day comments) + composer ──
        let mut journal_items: Vec<ListItem> = data
            .comments
            .iter()
            .map(|comment| {
                let time = comment.timestamp.get(11..16).unwrap_or("").to_string();
                ListItem::new(Line::from(vec![
                    Span::styled(time, Style::default().fg(Color::DarkGray)),
                    Span::raw(" "),
                    Span::styled(
                        display_title(&comment.item),
                        Style::default().fg(Color::Magenta),
                    ),
                    Span::raw(" "),
                    Span::raw(comment.content.replace('\n', " ")),
                ]))
            })
            .collect();
        if let Some(input) = &self.input {
            journal_items.push(ListItem::new(Line::from(vec![
                Span::styled("✎ ", Style::default().fg(Color::Yellow)),
                Span::raw(input.clone()),
                Span::styled("▏", Style::default().fg(Color::Yellow)),
            ])));
        } else if data.comments.is_empty() {
            journal_items.push(ListItem::new(Span::styled(
                "press i to journal",
                Style::default().fg(Color::DarkGray),
            )));
        }
        self.section(
            frame,
            journal_area,
            " Journal ",
            Section::Journal,
            journal_items,
            self.journal,
        );
    }
}

impl TodayState {
    /// Render one dashboard section: cyan border + selection highlight when
    /// focused, dim and static otherwise.
    fn section(
        &self,
        frame: &mut Frame,
        area: Rect,
        title: &str,
        section: Section,
        items: Vec<ListItem>,
        cursor: Select,
    ) {
        let focused = self.focus == section && self.input.is_none();
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title.to_string())
            .border_style(border(focused));
        let mut list = List::new(items).block(block);
        let mut state = ListState::default();
        if focused {
            list = list.highlight_style(Style::default().add_modifier(Modifier::REVERSED));
            state.select(Some(cursor.index));
        }
        frame.render_stateful_widget(list, area, &mut state);
    }
}

/// Build list items from `iter`, or a single dim hint when it is empty.
fn list_or_hint<'a>(iter: impl Iterator<Item = ListItem<'a>>, hint: &'a str) -> Vec<ListItem<'a>> {
    let items: Vec<ListItem> = iter.collect();
    if items.is_empty() {
        vec![ListItem::new(Span::styled(
            hint,
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        items
    }
}
