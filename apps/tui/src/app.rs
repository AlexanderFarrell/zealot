use std::time::Instant;

use chrono::{Duration, Local, NaiveDate};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use zealot_domain::item::{ItemDto, SearchResultDto, SearchScope};
use zealot_domain::repeat::RepeatEntryDto;
use zealot_domain::rule::RuleDto;

use crate::msg::{DayData, LinkKind, Load, Msg};
use crate::net::Net;

pub const HABIT_STATUSES: [&str; 4] = ["Not Complete", "Complete", "Skip", "Alternate"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Today,
    Browse,
    Search,
    Viewer,
    Habits,
    Rules,
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
        }
    }
}

// ─── Per-screen state ────────────────────────────────────────────────────────

pub struct TodayState {
    pub date: NaiveDate,
    pub data: Load<DayData>,
    pub selected_habit: usize,
    /// Journal input line, when `i` is active.
    pub input: Option<String>,
    pub generation: u64,
}

pub struct TreeNode {
    pub item: ItemDto,
    pub depth: usize,
    pub expanded: bool,
    pub children_loaded: bool,
}

#[derive(Default)]
pub struct BrowseState {
    pub nodes: Vec<TreeNode>,
    /// Index into the *visible* node list.
    pub selected: usize,
    pub loading: bool,
    pub started: bool,
    pub generation: u64,
}

pub struct SearchState {
    pub query: String,
    pub scope: SearchScope,
    pub regex: bool,
    pub results: Load<Vec<SearchResultDto>>,
    pub selected: usize,
    /// Set when the query changes; search fires once it is >250ms old.
    pub dirty_at: Option<Instant>,
    pub generation: u64,
}

#[derive(Default)]
pub struct ViewerState {
    pub item: Option<ItemDto>,
    pub rendered: String,
    pub scroll: u16,
    pub back_stack: Vec<i64>,
    /// Overlay list of backlinks/related/children to jump through.
    pub links: Option<(LinkKind, Load<Vec<ItemDto>>, usize)>,
}

pub struct HabitsState {
    pub end: NaiveDate,
    pub days: i64,
    pub entries: Load<Vec<RepeatEntryDto>>,
    pub row: usize,
    pub col: usize,
    pub generation: u64,
}

#[derive(Default)]
pub struct RulesState {
    pub rules: Load<Vec<RuleDto>>,
    pub selected: usize,
    pub started: bool,
    pub generation: u64,
}

pub struct PaletteState {
    pub query: String,
    pub selected: usize,
}

#[derive(Debug, Clone)]
pub enum PaletteAction {
    Goto(Screen),
    OpenByTitle(String),
    Refresh,
    Quit,
}

impl PaletteAction {
    pub fn label(&self) -> String {
        match self {
            PaletteAction::Goto(screen) => format!("Go to {}", screen.title()),
            PaletteAction::OpenByTitle(title) => format!("Open item '{title}'"),
            PaletteAction::Refresh => "Refresh current screen".into(),
            PaletteAction::Quit => "Quit".into(),
        }
    }
}

/// A request to suspend the TUI and open $EDITOR.
pub enum EditRequest {
    ItemContent { item_id: i64, content: String },
    RuleScript { rule_id: i64, script: String },
}

// ─── App ─────────────────────────────────────────────────────────────────────

pub struct App {
    pub net: Net,
    pub screen: Screen,
    pub today: TodayState,
    pub browse: BrowseState,
    pub search: SearchState,
    pub viewer: ViewerState,
    pub habits: HabitsState,
    pub rules: RulesState,
    pub palette: Option<PaletteState>,
    pub show_help: bool,
    pub toast: Option<(String, bool, Instant)>, // text, is_error, shown at
    pub should_quit: bool,
    pub pending_edit: Option<EditRequest>,
    pub journal_item: Option<String>,
    pub profile_label: String,
    pub generation: u64,
    pub last_auto_refresh: Instant,
}

impl App {
    pub fn new(net: Net, profile_label: String, journal_item: Option<String>) -> Self {
        let today = Local::now().date_naive();
        let mut app = Self {
            net,
            screen: Screen::Today,
            today: TodayState {
                date: today,
                data: Load::Idle,
                selected_habit: 0,
                input: None,
                generation: 0,
            },
            browse: BrowseState::default(),
            search: SearchState {
                query: String::new(),
                scope: SearchScope::Title,
                regex: false,
                results: Load::Idle,
                selected: 0,
                dirty_at: None,
                generation: 0,
            },
            viewer: ViewerState::default(),
            habits: HabitsState {
                end: today,
                days: 14,
                entries: Load::Idle,
                row: 0,
                col: 13,
                generation: 0,
            },
            rules: RulesState::default(),
            palette: None,
            show_help: false,
            toast: None,
            should_quit: false,
            pending_edit: None,
            journal_item,
            profile_label,
            generation: 0,
            last_auto_refresh: Instant::now(),
        };
        app.refresh_today();
        app
    }

    fn next_generation(&mut self) -> u64 {
        self.generation += 1;
        self.generation
    }

    pub fn toast(&mut self, text: impl Into<String>, is_error: bool) {
        self.toast = Some((text.into(), is_error, Instant::now()));
    }

    // ─── Data loading ────────────────────────────────────────────────────────

    pub fn refresh_today(&mut self) {
        let generation = self.next_generation();
        self.today.generation = generation;
        self.today.data = Load::Loading;
        self.net.load_day(self.today.date, generation);
    }

    pub fn refresh_browse(&mut self) {
        let generation = self.next_generation();
        self.browse.generation = generation;
        self.browse.loading = true;
        self.browse.started = true;
        self.net.load_roots(generation);
    }

    pub fn refresh_habits(&mut self) {
        let generation = self.next_generation();
        self.habits.generation = generation;
        self.habits.entries = Load::Loading;
        let start = self.habits.end - Duration::days(self.habits.days - 1);
        self.net.load_habits_range(start, self.habits.end, generation);
    }

    pub fn refresh_rules(&mut self) {
        let generation = self.next_generation();
        self.rules.generation = generation;
        self.rules.started = true;
        self.rules.rules = Load::Loading;
        self.net.load_rules(generation);
    }

    pub fn refresh_current(&mut self) {
        match self.screen {
            Screen::Today => self.refresh_today(),
            Screen::Browse => self.refresh_browse(),
            Screen::Search => self.fire_search(),
            Screen::Viewer => {
                if let Some(item) = &self.viewer.item {
                    self.net.open_item(item.item_id);
                }
            }
            Screen::Habits => self.refresh_habits(),
            Screen::Rules => self.refresh_rules(),
        }
    }

    pub fn goto(&mut self, screen: Screen) {
        self.screen = screen;
        match screen {
            Screen::Browse if !self.browse.started => self.refresh_browse(),
            Screen::Habits if matches!(self.habits.entries, Load::Idle) => self.refresh_habits(),
            Screen::Rules if !self.rules.started => self.refresh_rules(),
            _ => {}
        }
    }

    fn fire_search(&mut self) {
        if self.search.query.trim().is_empty() {
            self.search.results = Load::Idle;
            return;
        }
        let generation = self.next_generation();
        self.search.generation = generation;
        self.search.results = Load::Loading;
        self.search.dirty_at = None;
        self.net.search(
            self.search.query.clone(),
            self.search.scope.clone(),
            self.search.regex,
            generation,
        );
    }

    // ─── Tick ────────────────────────────────────────────────────────────────

    pub fn on_tick(&mut self) {
        // Debounced live search.
        if let Some(at) = self.search.dirty_at {
            if at.elapsed().as_millis() > 250 {
                self.fire_search();
            }
        }
        // Expire toasts.
        if let Some((_, _, at)) = &self.toast {
            if at.elapsed().as_secs() > 4 {
                self.toast = None;
            }
        }
        // Auto-refresh Today every 60s.
        if self.screen == Screen::Today
            && self.today.input.is_none()
            && self.last_auto_refresh.elapsed().as_secs() >= 60
        {
            self.last_auto_refresh = Instant::now();
            self.refresh_today();
        }
    }

    // ─── Net message handling ────────────────────────────────────────────────

    pub fn on_msg(&mut self, msg: Msg) {
        match msg {
            Msg::Day { generation, data } => {
                if generation != self.today.generation {
                    return;
                }
                self.today.data = match data {
                    Ok(data) => {
                        let habit_count = data.habits.len();
                        if self.today.selected_habit >= habit_count && habit_count > 0 {
                            self.today.selected_habit = habit_count - 1;
                        }
                        Load::Loaded(data)
                    }
                    Err(e) => Load::Error(e),
                };
            }
            Msg::Roots { generation, items } => {
                if generation != self.browse.generation {
                    return;
                }
                self.browse.loading = false;
                match items {
                    Ok(items) => {
                        self.browse.nodes = items
                            .into_iter()
                            .map(|item| TreeNode {
                                item,
                                depth: 0,
                                expanded: false,
                                children_loaded: false,
                            })
                            .collect();
                        self.browse.selected = 0;
                    }
                    Err(e) => self.toast(format!("browse: {e}"), true),
                }
            }
            Msg::Children {
                generation,
                parent_id,
                items,
            } => {
                if generation != self.browse.generation {
                    return;
                }
                self.browse.loading = false;
                match items {
                    Ok(items) => self.insert_children(parent_id, items),
                    Err(e) => self.toast(format!("children: {e}"), true),
                }
            }
            Msg::SearchResults {
                generation,
                results,
            } => {
                if generation != self.search.generation {
                    return;
                }
                self.search.results = match results {
                    Ok(results) => {
                        self.search.selected = 0;
                        Load::Loaded(results)
                    }
                    Err(e) => Load::Error(e),
                };
            }
            Msg::OpenItem { item } => match item {
                Ok(item) => {
                    if let Some(current) = &self.viewer.item {
                        if self.screen == Screen::Viewer && current.item_id != item.item_id {
                            self.viewer.back_stack.push(current.item_id);
                        }
                    }
                    self.viewer.rendered = zealot_zscript::render(&item.content);
                    self.viewer.item = Some(*item);
                    self.viewer.scroll = 0;
                    self.viewer.links = None;
                    self.screen = Screen::Viewer;
                }
                Err(e) => self.toast(format!("open: {e}"), true),
            },
            Msg::Links { kind, items } => {
                if let Some((expected, slot, _)) = &mut self.viewer.links {
                    if *expected == kind {
                        *slot = match items {
                            Ok(items) => Load::Loaded(items),
                            Err(e) => Load::Error(e),
                        };
                    }
                }
            }
            Msg::HabitsRange {
                generation,
                entries,
            } => {
                if generation != self.habits.generation {
                    return;
                }
                self.habits.entries = match entries {
                    Ok(entries) => Load::Loaded(entries),
                    Err(e) => Load::Error(e),
                };
            }
            Msg::Rules { generation, rules } => {
                if generation != self.rules.generation {
                    return;
                }
                self.rules.rules = match rules {
                    Ok(rules) => {
                        if self.rules.selected >= rules.len() && !rules.is_empty() {
                            self.rules.selected = rules.len() - 1;
                        }
                        Load::Loaded(rules)
                    }
                    Err(e) => Load::Error(e),
                };
            }
            Msg::RuleRan { name, result } => {
                match result {
                    Ok(run) if run.success => {
                        self.toast(format!("'{name}' ran in {}ms", run.duration_ms), false)
                    }
                    Ok(run) => self.toast(
                        format!(
                            "'{name}' failed: {}",
                            run.error.unwrap_or_else(|| "unknown error".into())
                        ),
                        true,
                    ),
                    Err(e) => self.toast(format!("'{name}': {e}"), true),
                }
                self.refresh_rules();
            }
            Msg::Done(result) => match result {
                Ok(text) => {
                    self.toast(text, false);
                    // Mutations usually affect the visible screen — refetch it.
                    self.refresh_current();
                }
                Err(e) => {
                    self.toast(e, true);
                    self.refresh_current();
                }
            },
        }
    }

    /// Splice freshly loaded children in after their parent node.
    fn insert_children(&mut self, parent_id: i64, items: Vec<ItemDto>) {
        let Some(pos) = self
            .browse
            .nodes
            .iter()
            .position(|n| n.item.item_id == parent_id)
        else {
            return;
        };
        let depth = self.browse.nodes[pos].depth + 1;
        self.browse.nodes[pos].children_loaded = true;
        self.browse.nodes[pos].expanded = true;
        let children: Vec<TreeNode> = items
            .into_iter()
            .map(|item| TreeNode {
                item,
                depth,
                expanded: false,
                children_loaded: false,
            })
            .collect();
        if children.is_empty() {
            self.toast("no children", false);
        }
        self.browse.nodes.splice(pos + 1..pos + 1, children);
    }

    /// Indices of nodes visible under current expansion state.
    pub fn visible_nodes(&self) -> Vec<usize> {
        let mut visible = Vec::new();
        let mut skip_deeper_than: Option<usize> = None;
        for (i, node) in self.browse.nodes.iter().enumerate() {
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

    // ─── Key handling ────────────────────────────────────────────────────────

    pub fn on_key(&mut self, key: KeyEvent) {
        // Overlays swallow keys first.
        if self.show_help {
            self.show_help = false;
            return;
        }
        if self.palette.is_some() {
            self.palette_key(key);
            return;
        }
        if self.today.input.is_some() && self.screen == Screen::Today {
            self.journal_input_key(key);
            return;
        }
        if self.viewer.links.is_some() && self.screen == Screen::Viewer {
            self.links_overlay_key(key);
            return;
        }

        // Global keys.
        match (key.code, key.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return;
            }
            (KeyCode::Char('p'), KeyModifiers::CONTROL) | (KeyCode::Char(':'), _) => {
                self.palette = Some(PaletteState {
                    query: String::new(),
                    selected: 0,
                });
                return;
            }
            (KeyCode::Char('?'), _) => {
                self.show_help = true;
                return;
            }
            (KeyCode::Char('1'), _) => return self.goto(Screen::Today),
            (KeyCode::Char('2'), _) => return self.goto(Screen::Browse),
            (KeyCode::Char('3'), _) => return self.goto(Screen::Search),
            (KeyCode::Char('4'), _) => return self.goto(Screen::Habits),
            (KeyCode::Char('5'), _) => return self.goto(Screen::Rules),
            (KeyCode::Char('/'), _) if self.screen != Screen::Search => {
                return self.goto(Screen::Search);
            }
            (KeyCode::Char('R'), _) => return self.refresh_current(),
            _ => {}
        }

        match self.screen {
            Screen::Today => self.today_key(key),
            Screen::Browse => self.browse_key(key),
            Screen::Search => self.search_key(key),
            Screen::Viewer => self.viewer_key(key),
            Screen::Habits => self.habits_key(key),
            Screen::Rules => self.rules_key(key),
        }
    }

    fn quit_or_back(&mut self) {
        if self.screen == Screen::Viewer {
            if let Some(previous) = self.viewer.back_stack.pop() {
                self.net.open_item(previous);
            } else {
                self.screen = Screen::Today;
            }
        } else {
            self.should_quit = true;
        }
    }

    fn today_key(&mut self, key: KeyEvent) {
        let habit_count = self
            .today
            .data
            .loaded()
            .map(|d| d.habits.len())
            .unwrap_or(0);
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit_or_back(),
            KeyCode::Char('[') => {
                self.today.date -= Duration::days(1);
                self.refresh_today();
            }
            KeyCode::Char(']') => {
                self.today.date += Duration::days(1);
                self.refresh_today();
            }
            KeyCode::Char('t') => {
                self.today.date = Local::now().date_naive();
                self.refresh_today();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if habit_count > 0 {
                    self.today.selected_habit = (self.today.selected_habit + 1) % habit_count;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if habit_count > 0 {
                    self.today.selected_habit =
                        (self.today.selected_habit + habit_count - 1) % habit_count;
                }
            }
            KeyCode::Char(' ') => self.cycle_selected_habit(),
            KeyCode::Char('i') => self.today.input = Some(String::new()),
            KeyCode::Enter => {
                // Open the selected habit's item.
                if let Some(data) = self.today.data.loaded() {
                    if let Some(entry) = data.habits.get(self.today.selected_habit) {
                        self.net.open_item(entry.item.item_id);
                    }
                }
            }
            _ => {}
        }
    }

    fn cycle_selected_habit(&mut self) {
        let date = self.today.date;
        let Some(data) = self.today.data.loaded_mut() else {
            return;
        };
        let Some(entry) = data.habits.get_mut(self.today.selected_habit) else {
            return;
        };
        let current = HABIT_STATUSES
            .iter()
            .position(|s| *s == entry.status)
            .unwrap_or(0);
        let next = HABIT_STATUSES[(current + 1) % HABIT_STATUSES.len()];
        entry.status = next.to_string(); // optimistic; Done() refetches
        self.net
            .set_habit_status(entry.item.item_id, date, next);
    }

    fn journal_input_key(&mut self, key: KeyEvent) {
        let Some(input) = &mut self.today.input else {
            return;
        };
        match key.code {
            KeyCode::Esc => self.today.input = None,
            KeyCode::Enter => {
                let text = input.trim().to_string();
                self.today.input = None;
                if text.is_empty() {
                    return;
                }
                match self.journal_item.clone() {
                    Some(title) => {
                        // Resolve title → id → comment, in one task chain.
                        let net = self.net.clone();
                        tokio::spawn(async move {
                            let item = if let Ok(id) = title.parse::<i64>() {
                                net.client.get_item(id).await
                            } else {
                                net.client.get_item_by_title(&title).await
                            };
                            match item {
                                Ok(item) => net.add_comment(item.item_id, text),
                                Err(e) => {
                                    let _ = net.tx.send(Msg::Done(Err(format!(
                                        "journal item '{title}': {e}"
                                    ))));
                                }
                            }
                        });
                    }
                    None => self.toast(
                        "no journal_item configured — set it in your zealot config profile",
                        true,
                    ),
                }
            }
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Char(c) => input.push(c),
            _ => {}
        }
    }

    fn browse_key(&mut self, key: KeyEvent) {
        let visible = self.visible_nodes();
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit_or_back(),
            KeyCode::Char('j') | KeyCode::Down => {
                if !visible.is_empty() {
                    self.browse.selected = (self.browse.selected + 1).min(visible.len() - 1);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.browse.selected = self.browse.selected.saturating_sub(1);
            }
            KeyCode::Char('g') => self.browse.selected = 0,
            KeyCode::Char('G') => {
                if !visible.is_empty() {
                    self.browse.selected = visible.len() - 1;
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if let Some(&idx) = visible.get(self.browse.selected) {
                    let node = &mut self.browse.nodes[idx];
                    if node.children_loaded {
                        node.expanded = true;
                    } else {
                        let id = node.item.item_id;
                        self.browse.loading = true;
                        self.net.load_children(id, self.browse.generation);
                    }
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if let Some(&idx) = visible.get(self.browse.selected) {
                    self.browse.nodes[idx].expanded = false;
                }
            }
            KeyCode::Enter => {
                if let Some(&idx) = visible.get(self.browse.selected) {
                    self.net.open_item(self.browse.nodes[idx].item.item_id);
                }
            }
            KeyCode::Char('e') => {
                if let Some(&idx) = visible.get(self.browse.selected) {
                    let item = &self.browse.nodes[idx].item;
                    self.pending_edit = Some(EditRequest::ItemContent {
                        item_id: item.item_id,
                        content: item.content.clone(),
                    });
                }
            }
            _ => {}
        }
    }

    fn search_key(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => self.goto(Screen::Today),
            (KeyCode::Tab, _) => {
                self.search.scope = match self.search.scope {
                    SearchScope::Title => SearchScope::Content,
                    SearchScope::Content => SearchScope::Heading,
                    SearchScope::Heading => SearchScope::Title,
                };
                self.search.dirty_at = Some(Instant::now());
            }
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                self.search.regex = !self.search.regex;
                self.search.dirty_at = Some(Instant::now());
            }
            (KeyCode::Down, _) => {
                if let Some(results) = self.search.results.loaded() {
                    if !results.is_empty() {
                        self.search.selected = (self.search.selected + 1).min(results.len() - 1);
                    }
                }
            }
            (KeyCode::Up, _) => self.search.selected = self.search.selected.saturating_sub(1),
            (KeyCode::Enter, _) => {
                if let Some(results) = self.search.results.loaded() {
                    if let Some(result) = results.get(self.search.selected) {
                        self.net.open_item(result.item.item_id);
                    }
                }
            }
            (KeyCode::Backspace, _) => {
                self.search.query.pop();
                self.search.dirty_at = Some(Instant::now());
            }
            (KeyCode::Char(c), _) => {
                self.search.query.push(c);
                self.search.dirty_at = Some(Instant::now());
            }
            _ => {}
        }
    }

    fn viewer_key(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q') | KeyCode::Esc | KeyCode::Backspace, _) => self.quit_or_back(),
            (KeyCode::Char('o'), KeyModifiers::CONTROL) => self.quit_or_back(),
            (KeyCode::Char('j') | KeyCode::Down, _) => {
                self.viewer.scroll = self.viewer.scroll.saturating_add(1);
            }
            (KeyCode::Char('k') | KeyCode::Up, _) => {
                self.viewer.scroll = self.viewer.scroll.saturating_sub(1);
            }
            (KeyCode::Char('d'), KeyModifiers::CONTROL) | (KeyCode::PageDown, _) => {
                self.viewer.scroll = self.viewer.scroll.saturating_add(15);
            }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) | (KeyCode::PageUp, _) => {
                self.viewer.scroll = self.viewer.scroll.saturating_sub(15);
            }
            (KeyCode::Char('g'), _) => self.viewer.scroll = 0,
            (KeyCode::Char('e'), _) => {
                if let Some(item) = &self.viewer.item {
                    self.pending_edit = Some(EditRequest::ItemContent {
                        item_id: item.item_id,
                        content: item.content.clone(),
                    });
                }
            }
            (KeyCode::Char('b'), _) => self.open_links(LinkKind::Backlinks),
            (KeyCode::Char('r'), _) => self.open_links(LinkKind::Related),
            (KeyCode::Char('c'), _) => self.open_links(LinkKind::Children),
            _ => {}
        }
    }

    fn open_links(&mut self, kind: LinkKind) {
        if let Some(item) = &self.viewer.item {
            self.viewer.links = Some((kind, Load::Loading, 0));
            self.net.load_links(item.item_id, kind);
        }
    }

    fn links_overlay_key(&mut self, key: KeyEvent) {
        let Some((_, slot, selected)) = &mut self.viewer.links else {
            return;
        };
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.viewer.links = None,
            KeyCode::Char('j') | KeyCode::Down => {
                if let Load::Loaded(items) = slot {
                    if !items.is_empty() {
                        *selected = (*selected + 1).min(items.len() - 1);
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => *selected = selected.saturating_sub(1),
            KeyCode::Enter => {
                if let Load::Loaded(items) = slot {
                    if let Some(item) = items.get(*selected) {
                        let id = item.item_id;
                        self.viewer.links = None;
                        self.net.open_item(id);
                    }
                }
            }
            _ => {}
        }
    }

    fn habits_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit_or_back(),
            KeyCode::Char('j') | KeyCode::Down => {
                let rows = self.habit_rows().len();
                if rows > 0 {
                    self.habits.row = (self.habits.row + 1).min(rows - 1);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => self.habits.row = self.habits.row.saturating_sub(1),
            KeyCode::Char('h') | KeyCode::Left => {
                self.habits.col = self.habits.col.saturating_sub(1);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.habits.col = (self.habits.col + 1).min(self.habits.days as usize - 1);
            }
            KeyCode::Char('[') => {
                self.habits.end -= Duration::days(7);
                self.refresh_habits();
            }
            KeyCode::Char(']') => {
                self.habits.end += Duration::days(7);
                self.refresh_habits();
            }
            KeyCode::Char('t') => {
                self.habits.end = Local::now().date_naive();
                self.refresh_habits();
            }
            KeyCode::Char(' ') => self.cycle_grid_habit(),
            _ => {}
        }
    }

    /// Distinct habit titles in the loaded range, in first-seen order.
    pub fn habit_rows(&self) -> Vec<(String, i64)> {
        let mut rows: Vec<(String, i64)> = Vec::new();
        if let Some(entries) = self.habits.entries.loaded() {
            for entry in entries {
                if !rows.iter().any(|(t, _)| *t == entry.item.title) {
                    rows.push((entry.item.title.clone(), entry.item.item_id));
                }
            }
        }
        rows
    }

    pub fn habit_grid_date(&self, col: usize) -> NaiveDate {
        self.habits.end - Duration::days(self.habits.days - 1 - col as i64)
    }

    fn cycle_grid_habit(&mut self) {
        let rows = self.habit_rows();
        let Some((_, item_id)) = rows.get(self.habits.row).cloned() else {
            return;
        };
        let date = self.habit_grid_date(self.habits.col);
        let date_str = date.format("%Y-%m-%d").to_string();
        let current = self
            .habits
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
        self.net.set_habit_status(item_id, date, next);
    }

    fn rules_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit_or_back(),
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(rules) = self.rules.rules.loaded() {
                    if !rules.is_empty() {
                        self.rules.selected = (self.rules.selected + 1).min(rules.len() - 1);
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.rules.selected = self.rules.selected.saturating_sub(1);
            }
            KeyCode::Char('r') | KeyCode::Enter => {
                let target = self.rules.rules.loaded().and_then(|rules| {
                    rules
                        .get(self.rules.selected)
                        .map(|rule| (rule.rule_id, rule.name.clone()))
                });
                if let Some((rule_id, name)) = target {
                    self.toast(format!("running '{name}'…"), false);
                    self.net.run_rule(rule_id, name);
                }
            }
            KeyCode::Char('e') => {
                if let Some(rules) = self.rules.rules.loaded() {
                    if let Some(rule) = rules.get(self.rules.selected) {
                        self.pending_edit = Some(EditRequest::RuleScript {
                            rule_id: rule.rule_id,
                            script: rule.script.clone(),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    // ─── Palette ─────────────────────────────────────────────────────────────

    pub fn palette_actions(&self) -> Vec<PaletteAction> {
        let query = self
            .palette
            .as_ref()
            .map(|p| p.query.to_lowercase())
            .unwrap_or_default();
        let mut actions = Vec::new();
        if !query.trim().is_empty() {
            actions.push(PaletteAction::OpenByTitle(
                self.palette.as_ref().unwrap().query.clone(),
            ));
        }
        for screen in [
            Screen::Today,
            Screen::Browse,
            Screen::Search,
            Screen::Habits,
            Screen::Rules,
        ] {
            actions.push(PaletteAction::Goto(screen));
        }
        actions.push(PaletteAction::Refresh);
        actions.push(PaletteAction::Quit);
        if query.trim().is_empty() {
            actions
        } else {
            actions
                .into_iter()
                .filter(|action| {
                    matches!(action, PaletteAction::OpenByTitle(_))
                        || action.label().to_lowercase().contains(&query)
                })
                .collect()
        }
    }

    fn palette_key(&mut self, key: KeyEvent) {
        let actions = self.palette_actions();
        let Some(palette) = &mut self.palette else {
            return;
        };
        match key.code {
            KeyCode::Esc => self.palette = None,
            KeyCode::Down => {
                if !actions.is_empty() {
                    palette.selected = (palette.selected + 1).min(actions.len() - 1);
                }
            }
            KeyCode::Up => palette.selected = palette.selected.saturating_sub(1),
            KeyCode::Backspace => {
                palette.query.pop();
                palette.selected = 0;
            }
            KeyCode::Enter => {
                let action = actions.get(palette.selected).cloned();
                self.palette = None;
                if let Some(action) = action {
                    match action {
                        PaletteAction::Goto(screen) => self.goto(screen),
                        PaletteAction::OpenByTitle(title) => {
                            self.net.open_item_by_title(title);
                        }
                        PaletteAction::Refresh => self.refresh_current(),
                        PaletteAction::Quit => self.should_quit = true,
                    }
                }
            }
            KeyCode::Char(c) => {
                palette.query.push(c);
                palette.selected = 0;
            }
            _ => {}
        }
    }
}
