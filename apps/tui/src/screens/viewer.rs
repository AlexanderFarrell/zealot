//! Viewer: the reusable single-item screen. Any screen that opens an item
//! lands here; a back-stack lets you drill through links and return.

use std::cell::Cell;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use zealot_domain::item::ItemDto;

use super::{Cx, EditRequest, Pane, Select, display_title};
use crate::msg::{LinkKind, Load};
use crate::net::Net;
use crate::ui::ansi::{BodyLink, LinkTarget, render_links};
use crate::ui::centered_rect;

#[derive(Default)]
pub struct ViewerState {
    pub item: Option<ItemDto>,
    /// Rendered content with link labels already styled; links are highlighted
    /// per-frame on top of this in `draw`.
    pub body: Text<'static>,
    /// Navigable links within `body`, in document order.
    pub nav_links: Vec<BodyLink>,
    /// Index into `nav_links` of the focused link, if any.
    pub selected_link: Option<usize>,
    pub scroll: u16,
    pub back_stack: Vec<i64>,
    /// Size of the content viewport from the last render, used to scroll the
    /// focused link into view. `(width, height)`.
    viewport: Cell<(u16, u16)>,
    /// Overlay list of backlinks/related/children to jump through.
    pub links: Option<(LinkKind, Load<Vec<ItemDto>>, Select)>,
}

impl ViewerState {
    /// Show a freshly loaded item. When `push_back` (i.e. we were already in
    /// the viewer), the outgoing item is remembered so Back can return to it.
    pub fn show(&mut self, item: ItemDto, push_back: bool) {
        if push_back
            && let Some(current) = &self.item
            && current.item_id != item.item_id
        {
            self.back_stack.push(current.item_id);
        }
        let (body, nav_links) = render_links(&zealot_zscript::render(&item.content));
        self.body = body;
        self.nav_links = nav_links;
        self.selected_link = None;
        self.item = Some(item);
        self.scroll = 0;
        self.links = None;
    }

    pub fn links_active(&self) -> bool {
        self.links.is_some()
    }

    /// Apply a links fetch result to the open overlay, if it still matches.
    pub fn set_links(&mut self, kind: LinkKind, items: Result<Vec<ItemDto>, String>) {
        if let Some((expected, slot, _)) = &mut self.links
            && *expected == kind
        {
            *slot = match items {
                Ok(items) => Load::Loaded(items),
                Err(e) => Load::Error(e),
            };
        }
    }

    fn open_links(&mut self, kind: LinkKind, net: &Net) {
        if let Some(item) = &self.item {
            self.links = Some((kind, Load::Loading, Select::default()));
            net.load_links(item.item_id, kind);
        }
    }

    /// Move the link cursor by `delta` (wrapping), scrolling the new target
    /// into view.
    fn cycle_link(&mut self, forward: bool) {
        let len = self.nav_links.len();
        if len == 0 {
            return;
        }
        let next = match self.selected_link {
            Some(i) if forward => (i + 1) % len,
            Some(i) => (i + len - 1) % len,
            None if forward => 0,
            None => len - 1,
        };
        self.selected_link = Some(next);
        self.scroll_to_link(next);
    }

    /// Adjust `scroll` so the focused link's line sits inside the viewport.
    fn scroll_to_link(&mut self, index: usize) {
        let Some(link) = self.nav_links.get(index) else {
            return;
        };
        let (width, height) = self.viewport.get();
        if width == 0 || height == 0 {
            return;
        }
        let row = self.visual_row_of_line(link.line, width);
        if row < self.scroll {
            self.scroll = row.saturating_sub(1);
        } else if row >= self.scroll + height {
            // Leave a couple of rows of trailing context.
            self.scroll = row.saturating_sub(height.saturating_sub(2));
        }
    }

    /// The first visual (post-wrap) row occupied by logical line `line_idx`,
    /// given the wrapping width — the coordinate space `Paragraph::scroll` uses.
    fn visual_row_of_line(&self, line_idx: usize, width: u16) -> u16 {
        let w = width.max(1) as usize;
        let rows: usize = self
            .body
            .lines
            .iter()
            .take(line_idx)
            .map(|line| (line.width().div_ceil(w)).max(1))
            .sum();
        rows.min(u16::MAX as usize) as u16
    }

    fn open_selected_link(&mut self, cx: &mut Cx) {
        let Some(index) = self.selected_link else {
            return;
        };
        let Some(link) = self.nav_links.get(index) else {
            return;
        };
        match &link.target {
            LinkTarget::Item(title) => cx.open_by_title(title.clone()),
            LinkTarget::Url(url) => {
                open_external(url);
                cx.toast(format!("opening {url}"), false);
            }
        }
    }

    /// Key handling while the links overlay is open. Returns without effect if
    /// the overlay isn't actually shown.
    pub fn links_key(&mut self, key: KeyEvent, cx: &mut Cx) {
        let Some((_, slot, sel)) = &mut self.links else {
            return;
        };
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.links = None,
            KeyCode::Char('j') | KeyCode::Down => {
                let len = slot.loaded().map(Vec::len).unwrap_or(0);
                sel.down(len);
            }
            KeyCode::Char('k') | KeyCode::Up => sel.up(),
            KeyCode::Enter => {
                if let Load::Loaded(items) = slot
                    && let Some(i) = sel.resolved(items.len())
                {
                    let id = items[i].item_id;
                    self.links = None;
                    cx.open(id);
                }
            }
            _ => {}
        }
    }
}

impl Pane for ViewerState {
    fn load(&mut self, net: &Net, _gen: u64) {
        if let Some(item) = &self.item {
            net.open_item(item.item_id);
        }
    }

    fn on_key(&mut self, key: KeyEvent, cx: &mut Cx) {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q') | KeyCode::Esc | KeyCode::Backspace, _) => cx.back(),
            (KeyCode::Char('o'), KeyModifiers::CONTROL) => cx.back(),
            (KeyCode::Tab, _) => self.cycle_link(true),
            (KeyCode::BackTab, _) => self.cycle_link(false),
            (KeyCode::Enter, _) => self.open_selected_link(cx),
            (KeyCode::Char('j') | KeyCode::Down, _) => {
                self.scroll = self.scroll.saturating_add(1);
            }
            (KeyCode::Char('k') | KeyCode::Up, _) => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            (KeyCode::Char('d'), KeyModifiers::CONTROL) | (KeyCode::PageDown, _) => {
                self.scroll = self.scroll.saturating_add(15);
            }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) | (KeyCode::PageUp, _) => {
                self.scroll = self.scroll.saturating_sub(15);
            }
            (KeyCode::Char('g'), _) => self.scroll = 0,
            (KeyCode::Char('e'), _) => {
                if let Some(item) = &self.item {
                    cx.edit(EditRequest::ItemContent {
                        item_id: item.item_id,
                        content: item.content.clone(),
                    });
                }
            }
            (KeyCode::Char('b'), _) => self.open_links(LinkKind::Backlinks, cx.net),
            (KeyCode::Char('r'), _) => self.open_links(LinkKind::Related, cx.net),
            (KeyCode::Char('c'), _) => self.open_links(LinkKind::Children, cx.net),
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let Some(item) = &self.item else {
            frame.render_widget(
                Paragraph::new("No item open. Find one via Browse (2) or Search (3).")
                    .style(Style::default().fg(Color::DarkGray)),
                area,
            );
            return;
        };

        let types = item
            .types
            .iter()
            .map(|t| t.name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let title = format!(
            " #{} {}{} ",
            item.item_id,
            display_title(item),
            if types.is_empty() {
                String::new()
            } else {
                format!(" · {types}")
            }
        );

        let mut block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Cyan));
        // Hint the link affordance only when there's something to navigate.
        if !self.nav_links.is_empty() {
            let hint = match self.selected_link {
                Some(i) => format!(" ⇥ link {}/{} · ↵ open ", i + 1, self.nav_links.len()),
                None => format!(" ⇥ {} links ", self.nav_links.len()),
            };
            block = block.title_bottom(Line::from(Span::styled(
                hint,
                Style::default().fg(Color::Magenta),
            )));
        }
        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Attribute strip on top, content below.
        let attrs: Vec<String> = match &item.attributes {
            serde_json::Value::Object(map) => map
                .iter()
                .filter(|(k, _)| *k != "Icon")
                .map(|(k, v)| {
                    let value = match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    format!("{k}: {value}")
                })
                .collect(),
            _ => Vec::new(),
        };

        let [attr_area, content_area] = Layout::vertical([
            Constraint::Length(if attrs.is_empty() { 0 } else { 1 }),
            Constraint::Min(1),
        ])
        .areas(inner);

        if !attrs.is_empty() {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    attrs.join("  ·  "),
                    Style::default().fg(Color::Yellow),
                ))),
                attr_area,
            );
        }

        // Overlay the focused-link highlight onto a clone of the base text.
        let mut body = self.body.clone();
        if let Some(index) = self.selected_link
            && let Some(link) = self.nav_links.get(index)
            && let Some(span) = body
                .lines
                .get_mut(link.line)
                .and_then(|line| line.spans.get_mut(link.span))
        {
            span.style = span.style.add_modifier(Modifier::REVERSED | Modifier::BOLD);
        }

        frame.render_widget(
            Paragraph::new(body)
                .wrap(Wrap { trim: false })
                .scroll((self.scroll, 0)),
            content_area,
        );
        self.viewport.set((content_area.width, content_area.height));

        if let Some((kind, slot, sel)) = &self.links {
            draw_links_overlay(frame, *kind, slot, sel);
        }
    }
}

/// Hand a URL to the platform's opener, best-effort.
fn open_external(url: &str) {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    let _ = std::process::Command::new(opener).arg(url).spawn();
}

fn draw_links_overlay(frame: &mut Frame, kind: LinkKind, slot: &Load<Vec<ItemDto>>, sel: &Select) {
    let title = match kind {
        LinkKind::Backlinks => " Backlinks ",
        LinkKind::Related => " Related ",
        LinkKind::Children => " Children ",
    };
    let area = centered_rect(60, 18, frame.area());
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Cyan));

    match slot {
        Load::Loaded(items) if items.is_empty() => frame.render_widget(
            Paragraph::new("None. (Esc to close)")
                .style(Style::default().fg(Color::DarkGray))
                .block(block),
            area,
        ),
        Load::Loaded(items) => {
            let list_items: Vec<ListItem> = items
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
                .collect();
            let mut state = ListState::default().with_selected(sel.resolved(items.len()));
            frame.render_stateful_widget(
                List::new(list_items)
                    .block(block)
                    .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
                area,
                &mut state,
            );
        }
        Load::Error(e) => frame.render_widget(
            Paragraph::new(format!("Error: {e}"))
                .style(Style::default().fg(Color::Red))
                .block(block),
            area,
        ),
        _ => frame.render_widget(
            Paragraph::new("Loading…")
                .style(Style::default().fg(Color::DarkGray))
                .block(block),
            area,
        ),
    }
}
