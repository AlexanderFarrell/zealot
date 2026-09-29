pub mod table;

// Styling + the ZealotScript renderer live in the shared zealot-zscript crate
// (also used by the TUI). Re-exported so command modules keep short paths.
pub use zealot_zscript::style::*;

/// Alias module so call sites can say `zscript::render(...)`.
pub mod zscript {
    pub use zealot_zscript::render::render;
}

/// Print any serializable value as pretty JSON (the `--json` contract).
pub fn print_json<T: serde::Serialize>(value: &T) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
