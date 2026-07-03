//! ZealotScript → ANSI terminal rendering, shared by the CLI and TUI.
//!
//! The renderer emits ANSI escape sequences; consumers that need structured
//! styles (the TUI) can parse the ANSI back into spans. Color output is
//! controlled by a process-global flag so piped/plain output stays clean.

pub mod render;
pub mod style;

pub use render::render;
pub use style::{set_color_enabled, visible_width};
