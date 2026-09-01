//! The application core: shared state, the async-message router, the tick
//! handler, and global/dispatch key routing. Per-screen behaviour lives in
//! [`crate::screens`]; the app only owns each screen's state and the glue
//! between them (command palette, help overlay, toasts, `$EDITOR` handoff).

use std::time::Instant;

use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::msg::{Load, Msg};
use crate::net::Net;
use crate::screens::browse::BrowseState;
use crate::screens::habits::HabitsState;
use crate::screens::random::RandomState;
use crate::screens::rules::RulesState;
use crate::screens::search::SearchState;
use crate::screens::today::TodayState;
use crate::screens::viewer::ViewerState;
use crate::screens::{Action, Cx, EditRequest, Pane, Screen};

pub const HABIT_STATUSES: [&str; 4] = ["Not Complete", "Complete", "Skip", "Alternate"];

// ─── Command palette ─────────────────────────────────────────────────────────

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
    pub random: RandomState,
    pub palette: Option<PaletteState>,
    pub show_help: bool,
    pub toast: Option<(String, bool, Instant)>, // text, is_error, shown at
    pub should_quit: bool,
    pub pending_edit: Option<EditRequest>,
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
            today: TodayState::new(today, journal_item),
            browse: BrowseState::default(),
            search: SearchState::default(),
            viewer: ViewerState::default(),
            habits: HabitsState::new(today),
            rules: RulesState::default(),
            random: RandomState::default(),
            palette: None,
            show_help: false,
            toast: None,
            should_quit: false,
            pending_edit: None,
            profile_label,
            generation: 0,
            last_auto_refresh: Instant::now(),
        };
        app.reload();
        app
    }

    fn next_generation(&mut self) -> u64 {
        self.generation += 1;
        self.generation
    }

    pub fn toast(&mut self, text: impl Into<String>, is_error: bool) {
        self.toast = Some((text.into(), is_error, Instant::now()));
    }

    // ─── Navigation & loading ────────────────────────────────────────────────

    /// Re-fetch the current screen's data with a fresh generation stamp.
    fn reload(&mut self) {
        let gen_id = self.next_generation();
        let net = self.net.clone();
        match self.screen {
            Screen::Today => self.today.load(&net, gen_id),
            Screen::Browse => self.browse.load(&net, gen_id),
            Screen::Search => self.search.load(&net, gen_id),
            Screen::Viewer => self.viewer.load(&net, gen_id),
            Screen::Habits => self.habits.load(&net, gen_id),
            Screen::Rules => self.rules.load(&net, gen_id),
            Screen::Random => self.random.load(&net, gen_id),
        }
    }

    fn goto(&mut self, screen: Screen) {
        self.screen = screen;
        let needs_load = match screen {
            Screen::Browse => !self.browse.started,
            Screen::Habits => matches!(self.habits.entries, Load::Idle),
            Screen::Rules => !self.rules.started,
            Screen::Random => !self.random.started,
            _ => false,
        };
        if needs_load {
            self.reload();
        }
    }

    /// Viewer: step back through the drill-down stack, else fall back to Today.
    /// Anywhere else: quit.
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

    fn apply(&mut self, action: Action) {
        match action {
            Action::Goto(screen) => self.goto(screen),
            Action::Back => self.quit_or_back(),
            Action::Reload => self.reload(),
            Action::Toast(text, is_error) => self.toast(text, is_error),
            Action::Edit(request) => self.pending_edit = Some(request),
        }
    }

    // ─── Tick ────────────────────────────────────────────────────────────────

    pub fn on_tick(&mut self) {
        // Debounced live search.
        if self
            .search
            .dirty_at
            .is_some_and(|at| at.elapsed().as_millis() > 250)
        {
            let gen_id = self.next_generation();
            let net = self.net.clone();
            self.search.fire(&net, gen_id);
        }
        // Expire toasts.
        if let Some((_, _, at)) = &self.toast
            && at.elapsed().as_secs() > 4
        {
            self.toast = None;
        }
        // Auto-refresh Today every 60s while idle.
        if self.screen == Screen::Today
            && self.today.input.is_none()
            && self.last_auto_refresh.elapsed().as_secs() >= 60
        {
            self.last_auto_refresh = Instant::now();
            self.reload();
        }
    }

    // ─── Async message router ────────────────────────────────────────────────

    pub fn on_msg(&mut self, msg: Msg) {
        match msg {
            Msg::Day { generation, data } => {
                if generation != self.today.generation {
                    return;
                }
                self.today.data = match data {
                    Ok(data) => Load::Loaded(data),
                    Err(e) => Load::Error(e),
                };
                self.today.clamp_cursors();
            }
            Msg::Roots { generation, items } => {
                if generation != self.browse.tree.generation {
                    return;
                }
                self.browse.tree.loading = false;
                match items {
                    Ok(items) => self
                        .browse
                        .tree
                        .set_roots(items.into_iter().map(|i| (i, None)).collect()),
                    Err(e) => self.toast(format!("browse: {e}"), true),
                }
            }
            Msg::Children {
                generation,
                parent_id,
                items,
            } => {
                // Children are routed to whichever tree fired them; generations
                // are globally unique, so at most one matches.
                let tree = if generation == self.browse.tree.generation {
                    Some(&mut self.browse.tree)
                } else if generation == self.search.tree.generation {
                    Some(&mut self.search.tree)
                } else if generation == self.random.tree.generation {
                    Some(&mut self.random.tree)
                } else {
                    None
                };
                let Some(tree) = tree else { return };
                match items {
                    Ok(items) => {
                        if tree.insert_children(parent_id, items) {
                            self.toast("no children", false);
                        }
                    }
                    Err(e) => {
                        tree.loading = false;
                        self.toast(format!("children: {e}"), true);
                    }
                }
            }
            Msg::Random { generation, items } => {
                if generation != self.random.tree.generation {
                    return;
                }
                self.random.status = match items {
                    Ok(items) => {
                        let count = items.len();
                        self.random
                            .tree
                            .set_roots(items.into_iter().map(|i| (i, None)).collect());
                        Load::Loaded(count)
                    }
                    Err(e) => Load::Error(e),
                };
            }
            Msg::SearchResults {
                generation,
                results,
            } => {
                if generation != self.search.generation {
                    return;
                }
                self.search.status = match results {
                    Ok(results) => {
                        let count = results.len();
                        self.search.tree.set_roots(
                            results
                                .into_iter()
                                .map(|r| (r.item, r.snippet.map(|s| s.replace('\n', " "))))
                                .collect(),
                        );
                        Load::Loaded(count)
                    }
                    Err(e) => Load::Error(e),
                };
            }
            Msg::OpenItem { item } => match item {
                Ok(item) => {
                    let push_back = self.screen == Screen::Viewer;
                    self.viewer.show(*item, push_back);
                    self.screen = Screen::Viewer;
                }
                Err(e) => self.toast(format!("open: {e}"), true),
            },
            Msg::Links { kind, items } => self.viewer.set_links(kind, items),
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
                        self.rules.sel.clamp(rules.len());
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
                let gen_id = self.next_generation();
                let net = self.net.clone();
                self.rules.load(&net, gen_id);
            }
            Msg::Done(result) => {
                match result {
                    Ok(text) => self.toast(text, false),
                    Err(e) => self.toast(e, true),
                }
                // Mutations usually affect the visible screen — refetch it.
                self.reload();
            }
        }
    }

    // ─── Key routing ─────────────────────────────────────────────────────────

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

        // Screen-owned modal overlays (journal composer, links list) route to
        // their screen with a Cx before global keys apply.
        let mut actions = Vec::new();
        let modal = {
            let mut cx = Cx::new(&self.net, &mut actions);
            if self.screen == Screen::Today && self.today.input.is_some() {
                self.today.journal_input_key(key, &mut cx);
                true
            } else if self.screen == Screen::Viewer && self.viewer.links_active() {
                self.viewer.links_key(key, &mut cx);
                true
            } else {
                false
            }
        };
        if modal {
            self.apply_all(actions);
            return;
        }

        // Global keys.
        match (key.code, key.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => return self.should_quit = true,
            (KeyCode::Char('p'), KeyModifiers::CONTROL) | (KeyCode::Char(':'), _) => {
                self.palette = Some(PaletteState {
                    query: String::new(),
                    selected: 0,
                });
                return;
            }
            (KeyCode::Char('?'), _) => return self.show_help = true,
            (KeyCode::Char('1'), _) => return self.goto(Screen::Today),
            (KeyCode::Char('2'), _) => return self.goto(Screen::Browse),
            (KeyCode::Char('3'), _) => return self.goto(Screen::Search),
            (KeyCode::Char('4'), _) => return self.goto(Screen::Habits),
            (KeyCode::Char('5'), _) => return self.goto(Screen::Rules),
            (KeyCode::Char('6'), _) => return self.goto(Screen::Random),
            (KeyCode::Char('/'), _) if self.screen != Screen::Search => {
                return self.goto(Screen::Search);
            }
            (KeyCode::Char('R'), _) => return self.reload(),
            _ => {}
        }

        // Dispatch to the active screen.
        let mut actions = Vec::new();
        {
            let mut cx = Cx::new(&self.net, &mut actions);
            match self.screen {
                Screen::Today => self.today.on_key(key, &mut cx),
                Screen::Browse => self.browse.on_key(key, &mut cx),
                Screen::Search => self.search.on_key(key, &mut cx),
                Screen::Viewer => self.viewer.on_key(key, &mut cx),
                Screen::Habits => self.habits.on_key(key, &mut cx),
                Screen::Rules => self.rules.on_key(key, &mut cx),
                Screen::Random => self.random.on_key(key, &mut cx),
            }
        }
        self.apply_all(actions);
    }

    fn apply_all(&mut self, actions: Vec<Action>) {
        for action in actions {
            self.apply(action);
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
        for screen in Screen::NAV {
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
                        PaletteAction::OpenByTitle(title) => self.net.open_item_by_title(title),
                        PaletteAction::Refresh => self.reload(),
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
