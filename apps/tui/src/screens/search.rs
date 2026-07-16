//! Search: a live-filtering query box over an expandable tree of results.
//! Matches are the tree's roots; each can be drilled into like Browse, and
//! changing the query replaces the whole tree.

use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use zealot_domain::item::SearchScope;

use super::{Cx, Pane, Screen, Tree};
use crate::msg::Load;
use crate::net::Net;

pub struct SearchState {
    pub query: String,
    pub scope: SearchScope,
    pub regex: bool,
    /// Result rows, drillable via [`Tree`]. `status` tracks the query lifecycle.
    pub tree: Tree,
    /// `Loaded(count)` once results land; drives the empty/loading/error hints.
    pub status: Load<usize>,
    /// Set when the query changes; the app fires the search once it is >250ms old.
    pub dirty_at: Option<Instant>,
    pub generation: u64,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            query: String::new(),
            scope: SearchScope::Title,
            regex: false,
            tree: Tree::default(),
            status: Load::Idle,
            dirty_at: None,
            generation: 0,
        }
    }
}

impl SearchState {
    /// Actually run the query. Empty queries clear the results instead.
    pub fn fire(&mut self, net: &Net, gen_id: u64) {
        self.dirty_at = None;
        if self.query.trim().is_empty() {
            self.status = Load::Idle;
            self.tree.set_roots(Vec::new());
            return;
        }
        self.generation = gen_id;
        self.tree.generation = gen_id;
        self.status = Load::Loading;
        net.search(self.query.clone(), self.scope.clone(), self.regex, gen_id);
    }

    fn touch(&mut self) {
        self.dirty_at = Some(Instant::now());
    }
}

impl Pane for SearchState {
    fn load(&mut self, net: &Net, gen_id: u64) {
        self.fire(net, gen_id);
    }

    fn on_key(&mut self, key: KeyEvent, cx: &mut Cx) {
        // Letter keys feed the query, so navigation stays on the arrow keys.
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => cx.goto(Screen::Today),
            (KeyCode::Tab, _) => {
                self.scope = match self.scope {
                    SearchScope::Title => SearchScope::Content,
                    SearchScope::Content => SearchScope::Heading,
                    SearchScope::Heading => SearchScope::Title,
                };
                self.touch();
            }
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                self.regex = !self.regex;
                self.touch();
            }
            (KeyCode::Down, _) => self.tree.down(),
            (KeyCode::Up, _) => self.tree.up(),
            (KeyCode::Right, _) => self.tree.expand(cx.net),
            (KeyCode::Left, _) => self.tree.collapse(),
            (KeyCode::Enter, _) => {
                if let Some(item) = self.tree.selected_item() {
                    cx.open(item.item_id);
                }
            }
            (KeyCode::Backspace, _) => {
                self.query.pop();
                self.touch();
            }
            (KeyCode::Char(c), _) => {
                self.query.push(c);
                self.touch();
            }
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let [input_area, results_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).areas(area);

        let scope = match self.scope {
            SearchScope::Title => "title",
            SearchScope::Content => "content",
            SearchScope::Heading => "heading",
        };
        let regex = if self.regex { " · regex" } else { "" };

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("🔎 ", Style::default().fg(Color::Cyan)),
                Span::raw(&self.query),
                Span::styled("▏", Style::default().fg(Color::Cyan)),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Screen::Search.titled(&format!(
                        "Search — {scope}{regex} (Tab scope · Ctrl-R regex)"
                    )))
                    .border_style(Style::default().fg(Color::Cyan)),
            ),
            input_area,
        );

        let results_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        match &self.status {
            Load::Loaded(count) => self.tree.render_list(
                frame,
                results_area,
                results_block.title(format!(" {count} result(s) · → children · Enter open ")),
            ),
            Load::Loading => frame.render_widget(
                Paragraph::new("Searching…")
                    .style(Style::default().fg(Color::DarkGray))
                    .block(results_block),
                results_area,
            ),
            Load::Idle => frame.render_widget(
                Paragraph::new("Type to search your wiki.")
                    .style(Style::default().fg(Color::DarkGray))
                    .block(results_block),
                results_area,
            ),
            Load::Error(e) => frame.render_widget(
                Paragraph::new(format!("Error: {e}"))
                    .style(Style::default().fg(Color::Red))
                    .block(results_block),
                results_area,
            ),
        }
    }
}
