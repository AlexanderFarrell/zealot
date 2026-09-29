use zealot_client::types::RuleRunResultDto;
use zealot_domain::comment::CommentDto;
use zealot_domain::item::{ItemDto, SearchResultDto};
use zealot_domain::repeat::RepeatEntryDto;
use zealot_domain::rule::RuleDto;
use zealot_domain::time_block::TimeBlockDto;

/// Async slot state for screen data.
#[derive(Debug, Default)]
pub enum Load<T> {
    #[default]
    Idle,
    Loading,
    Loaded(T),
    Error(String),
}

impl<T> Load<T> {
    pub fn loaded(&self) -> Option<&T> {
        match self {
            Load::Loaded(value) => Some(value),
            _ => None,
        }
    }

    pub fn loaded_mut(&mut self) -> Option<&mut T> {
        match self {
            Load::Loaded(value) => Some(value),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct DayData {
    pub plan: Vec<ItemDto>,
    pub habits: Vec<RepeatEntryDto>,
    pub blocks: Vec<TimeBlockDto>,
    pub comments: Vec<CommentDto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkKind {
    Backlinks,
    Related,
    Children,
}

/// Results of async work, delivered back to the app loop.
#[derive(Debug)]
pub enum Msg {
    Day {
        generation: u64,
        data: Result<DayData, String>,
    },
    Roots {
        generation: u64,
        items: Result<Vec<ItemDto>, String>,
    },
    Children {
        generation: u64,
        parent_id: i64,
        items: Result<Vec<ItemDto>, String>,
    },
    Random {
        generation: u64,
        items: Result<Vec<ItemDto>, String>,
    },
    SearchResults {
        generation: u64,
        results: Result<Vec<SearchResultDto>, String>,
    },
    OpenItem {
        item: Result<Box<ItemDto>, String>,
    },
    Links {
        kind: LinkKind,
        items: Result<Vec<ItemDto>, String>,
    },
    HabitsRange {
        generation: u64,
        entries: Result<Vec<RepeatEntryDto>, String>,
    },
    Rules {
        generation: u64,
        rules: Result<Vec<RuleDto>, String>,
    },
    RuleRan {
        name: String,
        result: Result<RuleRunResultDto, String>,
    },
    /// Outcome of a fire-and-forget mutation; Ok toasts, Err warns.
    Done(Result<String, String>),
}
