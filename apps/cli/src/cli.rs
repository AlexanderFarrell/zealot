//! The `zealot` command tree.
//!
//! Noun-verb commands (`zealot item view …`) plus one-letter power aliases for
//! the highest-frequency actions: `s` search, `v` view, `a` add, `j` journal,
//! `d` day.

use clap::{Args, Parser, Subcommand};

use crate::args::{DateArg, ItemRef, TimeRange};

#[derive(Parser)]
#[command(
    name = "zealot",
    version,
    about = "Zealot from your terminal — wiki, planner, and habits at speed",
    propagate_version = true
)]
pub struct Cli {
    /// Config profile to use (multiple servers supported)
    #[arg(long, global = true)]
    pub profile: Option<String>,

    /// Emit raw JSON (for jq / scripting)
    #[arg(long, global = true)]
    pub json: bool,

    /// Disable colored output (NO_COLOR is also honored)
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Server URL override (else config / ZEALOT_URL)
    #[arg(long, global = true)]
    pub url: Option<String>,

    /// API key override (else config / ZEALOT_API_KEY)
    #[arg(long, global = true)]
    pub api_key: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Connect to a Zealot server and store an API key
    Login {
        /// Server URL (e.g. https://zealot.example.com); prompted if omitted
        url: Option<String>,
        /// Profile name to store the credentials under
        #[arg(long, default_value = "default")]
        name: String,
    },
    /// Revoke the stored API key and forget the profile's credentials
    Logout {
        /// Keep the key active server-side; only forget it locally
        #[arg(long)]
        keep_key: bool,
    },
    /// Show connection, account, and profile status
    Status,

    /// Work with wiki items
    #[command(subcommand)]
    Item(ItemCmd),

    /// View an item (shortcut for `item view`)
    #[command(visible_alias = "v")]
    View(ViewArgs),

    /// Quick-add an item (shortcut for `item new`)
    #[command(visible_alias = "a")]
    Add(NewArgs),

    /// Search items
    #[command(visible_alias = "s")]
    Search {
        term: String,
        /// Search inside item content instead of titles
        #[arg(long, short = 'c')]
        content: bool,
        /// Search headings
        #[arg(long)]
        heading: bool,
        /// Treat the term as a case-insensitive regex
        #[arg(long, short = 'r')]
        regex: bool,
        #[arg(long, short = 'n', default_value_t = 20)]
        limit: i64,
        #[arg(long, default_value_t = 0)]
        offset: i64,
        /// Fetch every page of results
        #[arg(long)]
        all: bool,
    },

    /// Filter items by attribute expressions, e.g. 'Status=Open' 'Due<=2026-07-10'
    Filter {
        /// KEY<OP>VALUE expressions; ops: = != > < >= <= ~ (ilike)
        #[arg(required = true)]
        exprs: Vec<String>,
        #[arg(long, short = 'n', default_value_t = 50)]
        limit: i64,
        #[arg(long, default_value_t = 0)]
        offset: i64,
    },

    /// Day dashboard: plan, habits, time blocks, journal
    #[command(visible_alias = "d")]
    Day {
        /// today, tomorrow, fri, +2, YYYY-MM-DD…
        date: Option<DateArg>,
        /// Include full item content
        #[arg(long)]
        full: bool,
    },
    /// Items planned for a week (e.g. 2026-W27; defaults to this week)
    Week { week: Option<String> },
    /// Items planned for a month
    Month {
        month: Option<u32>,
        year: Option<i32>,
    },
    /// Items planned for a year
    Year { year: Option<i32> },

    /// Habit tracking (repeat entries)
    #[command(subcommand)]
    Habit(HabitCmd),

    /// Time blocks
    #[command(subcommand)]
    Block(BlockCmd),

    /// Comments and journaling
    #[command(subcommand)]
    Comment(CommentCmd),

    /// Add a journal comment for right now (uses the profile's journal item)
    #[command(visible_alias = "j")]
    Journal {
        /// Comment text; reads stdin when omitted or '-'
        text: Vec<String>,
    },

    /// Item types
    #[command(subcommand, name = "type")]
    ItemType(TypeCmd),

    /// Attribute kinds
    #[command(subcommand)]
    Attr(AttrCmd),

    /// Automation rules (Lua)
    #[command(subcommand)]
    Rule(RuleCmd),

    /// Typed numeric statistics
    #[command(subcommand)]
    Statistic(StatisticCmd),

    /// Media files
    #[command(subcommand)]
    Media(MediaCmd),

    /// Raw API escape hatch: zealot api GET /item/recent
    Api {
        /// HTTP method (GET, POST, PATCH, PUT, DELETE)
        method: String,
        /// Path, e.g. /item/search?term=x
        path: String,
        /// JSON body; '-' or omitted-with-pipe reads stdin
        #[arg(long, short = 'b')]
        body: Option<String>,
    },

    /// Launch the full-screen TUI (zealot-tui)
    Tui,

    /// Generate shell completions
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[derive(Args)]
pub struct ViewArgs {
    /// Item id or title
    pub item: ItemRef,
    /// Print raw ZealotScript instead of rendering
    #[arg(long)]
    pub raw: bool,
    /// Show attributes only
    #[arg(long)]
    pub attrs: bool,
    /// Show metadata (id, types, attributes, links) with the content
    #[arg(long, short = 'm')]
    pub meta: bool,
}

#[derive(Args)]
pub struct NewArgs {
    /// Title of the new item
    pub title: String,
    /// Assign a type (repeatable)
    #[arg(long = "type", short = 't')]
    pub types: Vec<String>,
    /// Set an attribute KEY=VALUE (repeatable)
    #[arg(long = "attr", short = 'A')]
    pub attrs: Vec<String>,
    /// Set the parent item
    #[arg(long, short = 'p')]
    pub parent: Option<ItemRef>,
    /// Content; reads stdin when piped, or use --edit
    #[arg(long, short = 'c')]
    pub content: Option<String>,
    /// Open $EDITOR for the content
    #[arg(long, short = 'e')]
    pub edit: bool,
}

#[derive(Subcommand)]
pub enum ItemCmd {
    /// Show an item, rendered for the terminal
    View(ViewArgs),
    /// Create an item
    New(NewArgs),
    /// Edit an item's content in $EDITOR
    Edit {
        item: ItemRef,
        /// Also edit title/types/attributes as TOML frontmatter
        #[arg(long, short = 'f')]
        full: bool,
    },
    /// Append text to an item's content (log-style capture)
    Append {
        item: ItemRef,
        /// Text to append; reads stdin when omitted or '-'
        text: Vec<String>,
    },
    /// Delete an item
    Rm {
        item: ItemRef,
        /// Skip the confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// List root items, or items of a type
    Ls {
        /// Filter by type name
        #[arg(long = "type", short = 't')]
        item_type: Option<String>,
    },
    /// Recently updated items
    Recent {
        #[arg(long, short = 'n', default_value_t = 30)]
        limit: i64,
    },
    /// Random items (rediscover your wiki)
    Random {
        #[arg(long, short = 'n', default_value_t = 5)]
        count: usize,
    },
    /// Most viewed items
    Top {
        #[arg(long, short = 'n', default_value_t = 15)]
        limit: i64,
    },
    /// Children of an item
    Children { item: ItemRef },
    /// Related items
    Related { item: ItemRef },
    /// Items linking to this item
    Backlinks { item: ItemRef },
    /// Show or modify an item's attributes
    #[command(subcommand)]
    Attr(ItemAttrCmd),
    /// Assign or remove a type on an item
    #[command(subcommand)]
    Type(ItemTypeAssignCmd),
    /// Export an item to PDF or DOCX
    Export {
        item: ItemRef,
        #[arg(long, conflicts_with = "docx")]
        pdf: bool,
        #[arg(long)]
        docx: bool,
        /// Output file (defaults to the server-suggested name)
        #[arg(long, short = 'o')]
        out: Option<String>,
    },
    /// Rebuild the derived link index
    RebuildLinks,
}

#[derive(Subcommand)]
pub enum ItemAttrCmd {
    /// List an item's attributes
    Ls { item: ItemRef },
    /// Set attributes: zealot item attr set <ITEM> Status=Open Priority=2
    Set {
        item: ItemRef,
        #[arg(required = true)]
        pairs: Vec<String>,
    },
    /// Remove an attribute
    Rm { item: ItemRef, key: String },
    /// Rename an attribute key
    Rename {
        item: ItemRef,
        old_key: String,
        new_key: String,
    },
}

#[derive(Subcommand)]
pub enum ItemTypeAssignCmd {
    /// Assign a type to an item
    Add { item: ItemRef, type_name: String },
    /// Remove a type from an item
    Rm { item: ItemRef, type_name: String },
}

#[derive(Subcommand)]
pub enum HabitCmd {
    /// Habits for a day with statuses
    Ls { date: Option<DateArg> },
    /// Mark a habit complete
    Done {
        item: ItemRef,
        date: Option<DateArg>,
        #[arg(long, short = 'm')]
        comment: Option<String>,
    },
    /// Mark a habit skipped
    Skip {
        item: ItemRef,
        date: Option<DateArg>,
        #[arg(long, short = 'm')]
        comment: Option<String>,
    },
    /// Mark a habit done via an alternate
    Alt {
        item: ItemRef,
        date: Option<DateArg>,
        #[arg(long, short = 'm')]
        comment: Option<String>,
    },
    /// Reset a habit to not complete
    Undo {
        item: ItemRef,
        date: Option<DateArg>,
    },
    /// 7-day habit grid ending at a date
    Week { date: Option<DateArg> },
    /// All habit-tracked items
    Items,
}

#[derive(Subcommand)]
pub enum BlockCmd {
    /// Time blocks for a day
    Ls { date: Option<DateArg> },
    /// Add a time block: zealot block add "Deep Work" 9:00-10:30 [DATE]
    Add {
        item: ItemRef,
        time: TimeRange,
        date: Option<DateArg>,
        #[arg(long, short = 'm')]
        note: Option<String>,
    },
    /// Edit a time block
    Edit {
        block_id: i64,
        #[arg(long)]
        time: Option<TimeRange>,
        #[arg(long)]
        date: Option<DateArg>,
        #[arg(long, short = 'm')]
        note: Option<String>,
    },
    /// Delete a time block
    Rm { block_id: i64 },
}

#[derive(Subcommand)]
pub enum StatisticCmd {
    /// List Statistic items
    Items {
        #[arg(long)]
        parent: Option<ItemRef>,
    },
    /// List raw entries for a Statistic item
    Entries {
        item: ItemRef,
        #[arg(long)]
        start: Option<String>,
        #[arg(long)]
        end: Option<String>,
        #[arg(long, short = 'n', default_value_t = 50)]
        limit: i64,
        #[arg(long, default_value_t = 0)]
        offset: i64,
    },
    /// Show daily aggregate points
    Daily {
        item: ItemRef,
        #[arg(long)]
        start: Option<String>,
        #[arg(long)]
        end: Option<String>,
    },
    /// Show a period summary
    Summary {
        item: ItemRef,
        #[arg(long)]
        start: Option<String>,
        #[arg(long)]
        end: Option<String>,
    },
    /// Record a Statistic Entry
    Record {
        item: ItemRef,
        value: f64,
        #[arg(long)]
        at: Option<String>,
        #[arg(long)]
        related: Option<ItemRef>,
        #[arg(long, short = 'm')]
        comment: Option<String>,
    },
    /// Edit a Statistic Entry
    Edit {
        statistic_entry_id: i64,
        #[arg(long)]
        value: Option<f64>,
        #[arg(long)]
        at: Option<String>,
        #[arg(long, conflicts_with = "clear_related")]
        related: Option<ItemRef>,
        #[arg(long)]
        clear_related: bool,
        #[arg(long, short = 'm', conflicts_with = "clear_comment")]
        comment: Option<String>,
        #[arg(long)]
        clear_comment: bool,
    },
    /// Delete a Statistic Entry
    Rm { statistic_entry_id: i64 },
}

#[derive(Subcommand)]
pub enum CommentCmd {
    /// Add a comment to an item
    Add {
        item: ItemRef,
        /// Comment text; reads stdin when omitted or '-'
        text: Vec<String>,
        /// Timestamp (YYYY-MM-DD HH:MM:SS); defaults to now
        #[arg(long)]
        at: Option<String>,
    },
    /// List comments for a day or an item
    Ls {
        /// Day to list (default today) — or use --item
        #[arg(long, conflicts_with = "item")]
        day: Option<Option<DateArg>>,
        #[arg(long)]
        item: Option<ItemRef>,
    },
    /// Edit a comment's text
    Edit { comment_id: i64, text: String },
    /// Delete a comment
    Rm { comment_id: i64 },
}

#[derive(Subcommand)]
pub enum TypeCmd {
    /// List item types
    Ls {
        /// Include item counts
        #[arg(long)]
        counts: bool,
    },
    /// Show one item type
    View { name: String },
    /// Create an item type
    New {
        name: String,
        #[arg(long, short = 'd', default_value = "")]
        description: String,
        /// Required attribute key (repeatable)
        #[arg(long = "attr", short = 'A')]
        required_attributes: Vec<String>,
    },
    /// Delete an item type
    Rm {
        name: String,
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum AttrCmd {
    /// List attribute kinds
    Ls,
    /// Show one attribute kind
    View { key: String },
    /// Create an attribute kind
    New {
        key: String,
        /// text, integer, decimal, date, week, dropdown, boolean, list, item
        #[arg(long, short = 'b')]
        base: String,
        #[arg(long, short = 'd', default_value = "")]
        description: String,
        /// Config JSON (e.g. '{"values":["A","B"]}' for dropdown)
        #[arg(long, default_value = "{}")]
        config: String,
    },
    /// Delete an attribute kind
    Rm {
        key: String,
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum RuleCmd {
    /// List rules with status and last run
    Ls,
    /// Show a rule (id or name)
    View {
        rule: String,
        /// Print the Lua script
        #[arg(long)]
        script: bool,
    },
    /// Run a rule now and print its output
    Run { rule: String },
    /// Enable a rule
    Enable { rule: String },
    /// Disable a rule
    Disable { rule: String },
    /// Edit a rule's Lua script in $EDITOR
    Edit { rule: String },
}

#[derive(Subcommand)]
pub enum MediaCmd {
    /// List a media directory
    Ls { path: Option<String> },
    /// Download a file
    Get {
        path: String,
        #[arg(long, short = 'o')]
        out: Option<String>,
    },
    /// Upload a file
    Put {
        file: String,
        /// Destination directory (defaults to root)
        path: Option<String>,
    },
    /// Delete a file or folder
    Rm { path: String },
    /// Create a folder
    Mkdir { path: String },
    /// Rename a file or folder
    Mv { from: String, to: String },
}

#[cfg(test)]
mod tests {
    use super::{Cli, Command, StatisticCmd};
    use clap::Parser;

    #[test]
    fn parses_statistic_record_and_nullable_edit_commands() {
        let record = Cli::try_parse_from([
            "zealot",
            "statistic",
            "record",
            "#42",
            "82.5",
            "--at",
            "2026-08-13T08:30:00Z",
            "--comment",
            "Morning",
        ])
        .unwrap();
        assert!(matches!(
            record.command,
            Command::Statistic(StatisticCmd::Record { value, .. }) if value == 82.5
        ));

        let edit = Cli::try_parse_from([
            "zealot",
            "statistic",
            "edit",
            "7",
            "--clear-related",
            "--clear-comment",
        ])
        .unwrap();
        assert!(matches!(
            edit.command,
            Command::Statistic(StatisticCmd::Edit {
                statistic_entry_id: 7,
                clear_related: true,
                clear_comment: true,
                ..
            })
        ));
    }
}
