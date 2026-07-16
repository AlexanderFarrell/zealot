//! Random: a handful of random items to rediscover. Reuses the shared [`Tree`]
//! so each pick can be opened or drilled into; `r` rerolls a fresh set.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{Cx, Pane, Screen, Tree};
use crate::msg::Load;
use crate::net::Net;

pub struct RandomState {
    pub tree: Tree,
    /// `Loaded(count)` once picks land; drives the loading/error hints.
    pub status: Load<usize>,
    pub count: usize,
    pub started: bool,
    pub generation: u64,
}

impl Default for RandomState {
    fn default() -> Self {
        Self {
            tree: Tree::default(),
            status: Load::Idle,
            count: 12,
            started: false,
            generation: 0,
        }
    }
}

impl Pane for RandomState {
    fn load(&mut self, net: &Net, gen_id: u64) {
        self.started = true;
        self.generation = gen_id;
        self.tree.generation = gen_id;
        self.status = Load::Loading;
        net.load_random(self.count, gen_id);
    }

    fn on_key(&mut self, key: KeyEvent, cx: &mut Cx) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => cx.back(),
            KeyCode::Char('j') | KeyCode::Down => self.tree.down(),
            KeyCode::Char('k') | KeyCode::Up => self.tree.up(),
            KeyCode::Char('g') => self.tree.first(),
            KeyCode::Char('G') => self.tree.last(),
            KeyCode::Char('l') | KeyCode::Right => self.tree.expand(cx.net),
            KeyCode::Char('h') | KeyCode::Left => self.tree.collapse(),
            KeyCode::Char('r') => cx.reload(),
            KeyCode::Enter => {
                if let Some(item) = self.tree.selected_item() {
                    cx.open(item.item_id);
                }
            }
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Screen::Random.titled("Random (r reroll · → children · Enter open)"))
            .border_style(Style::default().fg(Color::Cyan));

        match &self.status {
            Load::Loaded(_) => self.tree.render_list(frame, area, block),
            Load::Loading => frame.render_widget(
                Paragraph::new("Shuffling…").style(Style::default().fg(Color::DarkGray)).block(block),
                area,
            ),
            Load::Idle => frame.render_widget(
                Paragraph::new("Press r for a fresh set.")
                    .style(Style::default().fg(Color::DarkGray))
                    .block(block),
                area,
            ),
            Load::Error(e) => frame.render_widget(
                Paragraph::new(format!("Error: {e}")).style(Style::default().fg(Color::Red)).block(block),
                area,
            ),
        }
    }
}
