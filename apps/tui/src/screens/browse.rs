//! Browse: a lazy-loading tree of the whole wiki with a live preview pane.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::{Cx, EditRequest, Pane, Screen, Tree};
use crate::net::Net;
use crate::ui::ansi::sgr_to_text;

#[derive(Default)]
pub struct BrowseState {
    pub tree: Tree,
    pub started: bool,
}

impl Pane for BrowseState {
    fn load(&mut self, net: &Net, gen_id: u64) {
        self.tree.generation = gen_id;
        self.tree.loading = true;
        self.started = true;
        net.load_roots(gen_id);
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
            KeyCode::Enter => {
                if let Some(item) = self.tree.selected_item() {
                    cx.open(item.item_id);
                }
            }
            KeyCode::Char('e') => {
                if let Some(item) = self.tree.selected_item() {
                    cx.edit(EditRequest::ItemContent {
                        item_id: item.item_id,
                        content: item.content.clone(),
                    });
                }
            }
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let [tree_area, preview_area] =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                .areas(area);

        let title =
            if self.tree.loading { Screen::Browse.titled("Items (loading…)") } else { Screen::Browse.titled("Items") };
        self.tree.render_list(
            frame,
            tree_area,
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        let preview_block = Block::default()
            .borders(Borders::ALL)
            .title(" Preview (Enter open · e edit) ")
            .border_style(Style::default().fg(Color::DarkGray));
        if let Some(item) = self.tree.selected_item() {
            let rendered = zealot_zscript::render(&item.content);
            frame.render_widget(
                Paragraph::new(sgr_to_text(&rendered))
                    .block(preview_block)
                    .wrap(Wrap { trim: false }),
                preview_area,
            );
        } else {
            frame.render_widget(Paragraph::new("No items.").block(preview_block), preview_area);
        }
    }
}
