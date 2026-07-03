//! Item references: every tool that identifies an item accepts a numeric ID
//! or an exact title, mirroring the CLI's `ItemRef` (apps/cli/src/args.rs).

use rmcp::ErrorData as McpError;
use zealot_domain::item::ItemDto;

use crate::tools::{ZealotServer, err_ctx};

pub enum ItemRef {
    Id(i64),
    Title(String),
}

impl ItemRef {
    pub fn parse(s: &str) -> Self {
        let t = s.trim();
        match t.strip_prefix('#').unwrap_or(t).parse::<i64>() {
            Ok(id) if id > 0 => ItemRef::Id(id),
            _ => ItemRef::Title(t.to_string()),
        }
    }
}

impl ZealotServer {
    /// Resolve "42", "#42", or an exact title to the full item.
    pub async fn resolve_item(&self, item: &str) -> Result<ItemDto, McpError> {
        match ItemRef::parse(item) {
            ItemRef::Id(id) => self
                .client
                .get_item(id)
                .await
                .map_err(|e| err_ctx(&format!("item #{id}"), e)),
            ItemRef::Title(t) => self
                .client
                .get_item_by_title(&t)
                .await
                .map_err(|e| err_ctx(&format!("item titled '{t}'"), e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_numeric_id() {
        assert!(matches!(ItemRef::parse("42"), ItemRef::Id(42)));
    }

    #[test]
    fn parses_hash_prefixed_id() {
        assert!(matches!(ItemRef::parse("#42"), ItemRef::Id(42)));
    }

    #[test]
    fn treats_negative_and_non_numeric_as_title() {
        assert!(matches!(ItemRef::parse("-1"), ItemRef::Title(_)));
        assert!(matches!(ItemRef::parse("My Note"), ItemRef::Title(_)));
    }
}
