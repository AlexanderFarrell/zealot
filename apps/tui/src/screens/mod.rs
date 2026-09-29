//! Screens: each is a self-contained module owning its state, key handling,
//! and rendering. A screen implements [`Pane`] and is driven by the app loop
//! through a lightweight [`Cx`] handle instead of a direct `&mut App`, so a
//! screen never needs to know about its siblings.

pub mod browse;
pub mod habits;
pub mod random;
pub mod rules;
pub mod search;
pub mod today;
pub mod viewer;

use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, ListState};
use zealot_domain::item::ItemDto;

use crate::net::Net;

/// The top-level screens the app can show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Today,
    Browse,
    Search,
    Viewer,
    Habits,
    Rules,
    Random,
}

impl Screen {
    pub fn title(&self) -> &'static str {
        match self {
            Screen::Today => "Today",
            Screen::Browse => "Browse",
            Screen::Search => "Search",
            Screen::Viewer => "Item",
            Screen::Habits => "Habits",
            Screen::Rules => "Rules",
            Screen::Random => "Random",
        }
    }

    /// The number key that jumps to this screen, shown in titles and the
    /// status bar. The viewer has no shortcut (you reach it by opening).
    pub fn key(&self) -> Option<char> {
        match self {
            Screen::Today => Some('1'),
            Screen::Browse => Some('2'),
            Screen::Search => Some('3'),
            Screen::Habits => Some('4'),
            Screen::Rules => Some('5'),
            Screen::Random => Some('6'),
            Screen::Viewer => None,
        }
    }

    /// Screens listed in the navigation legend / palette, in key order.
    pub const NAV: [Screen; 6] = [
        Screen::Today,
        Screen::Browse,
        Screen::Search,
        Screen::Habits,
        Screen::Rules,
        Screen::Random,
    ];

    /// Border title prefixed with this screen's shortcut, e.g. `[2] Items`.
    pub fn titled(&self, text: &str) -> String {
        match self.key() {
            Some(k) => format!(" [{k}] {text} "),
            None => format!(" {text} "),
        }
    }
}

/// A request to suspend the TUI and open `$EDITOR`.
pub enum EditRequest {
    ItemContent { item_id: i64, content: String },
    RuleScript { rule_id: i64, script: String },
}

/// An app-level effect a screen asks for. Screens can't touch the `App`
/// directly (they only get a `&mut Self`), so anything that reaches beyond a
/// screen's own state is expressed as an `Action` the app applies afterwards.
pub enum Action {
    Goto(Screen),
    /// Viewer: pop the back-stack, otherwise fall back to Today / quit.
    Back,
    /// Re-fetch the current screen's data.
    Reload,
    Toast(String, bool),
    Edit(EditRequest),
}

/// Handle passed to a screen's [`Pane::on_key`]. Exposes the async data layer
/// directly (fire-and-forget fetches flow back as `Msg`s) and queues
/// app-level [`Action`]s.
pub struct Cx<'a> {
    pub net: &'a Net,
    out: &'a mut Vec<Action>,
}

impl<'a> Cx<'a> {
    pub fn new(net: &'a Net, out: &'a mut Vec<Action>) -> Self {
        Self { net, out }
    }

    pub fn goto(&mut self, screen: Screen) {
        self.out.push(Action::Goto(screen));
    }
    pub fn back(&mut self) {
        self.out.push(Action::Back);
    }
    pub fn reload(&mut self) {
        self.out.push(Action::Reload);
    }
    pub fn toast(&mut self, text: impl Into<String>, is_error: bool) {
        self.out.push(Action::Toast(text.into(), is_error));
    }
    pub fn edit(&mut self, request: EditRequest) {
        self.out.push(Action::Edit(request));
    }
    /// Open an item in the viewer (by id). Result arrives as `Msg::OpenItem`.
    pub fn open(&mut self, item_id: i64) {
        self.net.open_item(item_id);
    }
    /// Open an item in the viewer by exact title. Result arrives as
    /// `Msg::OpenItem`; a miss comes back as an error toast.
    pub fn open_by_title(&mut self, title: String) {
        self.net.open_item_by_title(title);
    }
}

/// The uniform contract every screen implements.
pub trait Pane {
    /// (Re)fetch this screen's data. `gen_id` stamps the request so stale
    /// responses can be discarded; the screen stores it in its own field.
    fn load(&mut self, net: &Net, gen_id: u64);
    /// Handle a key press already filtered of global/overlay bindings.
    fn on_key(&mut self, key: KeyEvent, cx: &mut Cx);
    /// Render into `area`.
    fn draw(&self, frame: &mut Frame, area: Rect);
}

// ─── Shared list cursor ──────────────────────────────────────────────────────

/// A reusable selection cursor over a list of `len` items. Every screen used
/// to hand-roll the same `min`/`saturating_sub` clamping; this centralises it.
#[derive(Default, Clone, Copy)]
pub struct Select {
    pub index: usize,
}

impl Select {
    pub fn up(&mut self) {
        self.index = self.index.saturating_sub(1);
    }
    pub fn down(&mut self, len: usize) {
        if len > 0 {
            self.index = (self.index + 1).min(len - 1);
        }
    }
    pub fn first(&mut self) {
        self.index = 0;
    }
    pub fn last(&mut self, len: usize) {
        if len > 0 {
            self.index = len - 1;
        }
    }
    /// Keep the cursor in range after the backing list changes.
    pub fn clamp(&mut self, len: usize) {
        if len == 0 {
            self.index = 0;
        } else if self.index >= len {
            self.index = len - 1;
        }
    }
    /// The selected index, or `None` when the list is empty.
    pub fn resolved(&self, len: usize) -> Option<usize> {
        (len > 0).then(|| self.index.min(len - 1))
    }
}

// ─── Shared lazy tree ────────────────────────────────────────────────────────

/// One row of a [`Tree`]: an item plus its expansion state.
pub struct TreeNode {
    pub item: ItemDto,
    pub depth: usize,
    pub expanded: bool,
    pub children_loaded: bool,
    /// Optional dim secondary line, e.g. a search snippet on a root row.
    pub note: Option<String>,
}

impl TreeNode {
    fn new(item: ItemDto, depth: usize, note: Option<String>) -> Self {
        Self {
            item,
            depth,
            expanded: false,
            children_loaded: false,
            note,
        }
    }
}

/// A lazily-expanded item tree with a visible-row cursor. Shared by the Browse,
/// Search, and Random screens. Child fetches are fired through [`Net`] and come
/// back as `Msg::Children`, routed to the right tree by `generation`.
#[derive(Default)]
pub struct Tree {
    pub nodes: Vec<TreeNode>,
    /// Cursor into the *visible* node list.
    pub sel: Select,
    pub loading: bool,
    pub generation: u64,
}

impl Tree {
    /// Indices of nodes visible under the current expansion state.
    pub fn visible_nodes(&self) -> Vec<usize> {
        let mut visible = Vec::new();
        let mut skip_deeper_than: Option<usize> = None;
        for (i, node) in self.nodes.iter().enumerate() {
            if let Some(depth) = skip_deeper_than {
                if node.depth > depth {
                    continue;
                }
                skip_deeper_than = None;
            }
            visible.push(i);
            if !node.expanded {
                skip_deeper_than = Some(node.depth);
            }
        }
        visible
    }

    /// The backing-node index currently selected, if any.
    fn selected_node(&self) -> Option<usize> {
        let visible = self.visible_nodes();
        visible.get(self.sel.resolved(visible.len())?).copied()
    }

    /// The item under the cursor, if any.
    pub fn selected_item(&self) -> Option<&ItemDto> {
        self.selected_node().map(|i| &self.nodes[i].item)
    }

    /// Replace the tree with fresh root rows, each with an optional note.
    pub fn set_roots(&mut self, roots: Vec<(ItemDto, Option<String>)>) {
        self.nodes = roots
            .into_iter()
            .map(|(item, note)| TreeNode::new(item, 0, note))
            .collect();
        self.sel.first();
        self.loading = false;
    }

    /// Splice freshly loaded children in after their parent. Returns `true`
    /// when the parent turned out to have no children.
    pub fn insert_children(&mut self, parent_id: i64, items: Vec<ItemDto>) -> bool {
        self.loading = false;
        let Some(pos) = self.nodes.iter().position(|n| n.item.item_id == parent_id) else {
            return false;
        };
        let depth = self.nodes[pos].depth + 1;
        self.nodes[pos].children_loaded = true;
        self.nodes[pos].expanded = true;
        let empty = items.is_empty();
        let children: Vec<TreeNode> = items
            .into_iter()
            .map(|item| TreeNode::new(item, depth, None))
            .collect();
        self.nodes.splice(pos + 1..pos + 1, children);
        empty
    }

    pub fn up(&mut self) {
        self.sel.up();
    }
    pub fn down(&mut self) {
        self.sel.down(self.visible_nodes().len());
    }
    pub fn first(&mut self) {
        self.sel.first();
    }
    pub fn last(&mut self) {
        self.sel.last(self.visible_nodes().len());
    }

    /// Expand the selected node, lazily loading its children on first open.
    pub fn expand(&mut self, net: &Net) {
        if let Some(idx) = self.selected_node() {
            let node = &mut self.nodes[idx];
            if node.children_loaded {
                node.expanded = true;
            } else {
                let id = node.item.item_id;
                self.loading = true;
                net.load_children(id, self.generation);
            }
        }
    }

    /// Collapse the selected node.
    pub fn collapse(&mut self) {
        if let Some(idx) = self.selected_node() {
            self.nodes[idx].expanded = false;
        }
    }

    /// Render the tree as a list inside `block`.
    pub fn render_list(&self, frame: &mut Frame, area: Rect, block: Block) {
        let visible = self.visible_nodes();
        let items: Vec<ListItem> = visible
            .iter()
            .map(|&idx| {
                let node = &self.nodes[idx];
                let arrow = if node.expanded {
                    "▾ "
                } else if node.children_loaded {
                    "▸ "
                } else {
                    "› "
                };
                let indent = "  ".repeat(node.depth);
                let mut lines = vec![Line::from(vec![
                    Span::raw(indent.clone()),
                    Span::styled(arrow, Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("#{} ", node.item.item_id),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw(display_title(&node.item)),
                ])];
                if let Some(note) = &node.note {
                    lines.push(Line::from(Span::styled(
                        format!("{indent}    {}", note.replace('\n', " ")),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                ListItem::new(lines)
            })
            .collect();
        let mut state = ListState::default().with_selected(self.sel.resolved(visible.len()));
        frame.render_stateful_widget(
            List::new(items)
                .block(block)
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
            area,
            &mut state,
        );
    }
}

// ─── Shared rendering helpers ────────────────────────────────────────────────

/// Item title prefixed with its `Icon` attribute, like the other clients.
pub fn display_title(item: &ItemDto) -> String {
    if let Some(serde_json::Value::String(icon)) = item.attributes.get("Icon") {
        format!("{icon} {}", item.title)
    } else {
        item.title.clone()
    }
}

pub fn status_span(status: &str) -> Span<'static> {
    match status {
        "Complete" => Span::styled("✔", Style::default().fg(Color::Green)),
        "Skip" => Span::styled("↷", Style::default().fg(Color::Yellow)),
        "Alternate" => Span::styled("◆", Style::default().fg(Color::Cyan)),
        _ => Span::styled("·", Style::default().fg(Color::DarkGray)),
    }
}

pub fn format_clock(min: i32) -> String {
    format!("{}:{:02}", min / 60, min % 60)
}

/// Border colour convention: cyan when the pane/section is focused, dim
/// otherwise. Used by multi-section screens.
pub fn border(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}
