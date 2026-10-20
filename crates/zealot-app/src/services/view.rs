//! Saved Views: run, validate and inspect the item query held in a `Saved View` item.
//!
//! A view is an ordinary item typed `Saved View` whose content holds one fenced `zealot-view`
//! JSON block (see `zealot_domain::saved_view`). Running it goes through
//! [`ItemService::query_items`] under the caller's account, so a view can never reveal items
//! outside the caller's scope and is bounded by the same 100-per-page limit. Nothing is cached:
//! a view computes on read.

use std::{collections::HashMap, sync::Arc, time::Instant};

use serde_json::Value;
use zealot_domain::{
    account::Account,
    attribute::{Attribute, AttributeScalar},
    common::id::Id,
    item::{Item, ItemDto},
    query_tokens,
    saved_view::{
        self, ParsedView, SAVED_VIEW_TYPE, SavedViewError, ViewGroupDto, ViewInspectDto,
        ViewResultDto,
    },
};

use crate::clock;

use super::item::{ItemService, ItemServiceError};

/// Default page size when neither the caller nor the definition sets one.
const DEFAULT_LIMIT: i64 = 50;
/// Hard page cap, matching `ItemService::query_items`.
const MAX_LIMIT: i64 = 100;
/// Title reported for a run of an unsaved definition.
const UNSAVED_TITLE: &str = "Unsaved view";
/// Label for the group of items that lack the `group_by` attribute.
const NONE_LABEL: &str = "(none)";

/// Runs and explains Saved Views.
#[derive(Debug)]
pub struct ViewService {
    item_service: Arc<ItemService>,
}

/// Errors from running or inspecting a view.
#[derive(Debug, thiserror::Error)]
pub enum ViewServiceError {
    #[error("not found")]
    NotFound,
    /// The item is not a Saved View, its definition is unreadable, or its query is rejected
    #[error("{0}")]
    Invalid(String),
    #[error("{0}")]
    Item(ItemServiceError),
}

impl From<ItemServiceError> for ViewServiceError {
    fn from(err: ItemServiceError) -> Self {
        match err {
            ItemServiceError::NotFound => Self::NotFound,
            ItemServiceError::InvalidFilter(msg) | ItemServiceError::InvalidId(msg) => {
                Self::Invalid(msg)
            }
            ItemServiceError::Attribute(err) => Self::Invalid(err.to_string()),
            other => Self::Item(other),
        }
    }
}

impl From<SavedViewError> for ViewServiceError {
    fn from(err: SavedViewError) -> Self {
        Self::Invalid(err.to_string())
    }
}

impl ViewService {
    pub fn new(item_service: &Arc<ItemService>) -> Self {
        Self {
            item_service: item_service.clone(),
        }
    }

    /// Run the view `view_id`, returning one page of items and, with `group_by`, its groups.
    pub fn run(
        &self,
        view_id: &Id,
        limit: Option<i64>,
        offset: Option<i64>,
        account: &Account,
    ) -> Result<ViewResultDto, ViewServiceError> {
        let (item, parsed) = self.load(view_id, account)?;
        let mut result = self.execute(parsed, limit, offset, account)?;
        tracing::debug!(view = %item.title, total = result.total, "ran saved view");
        result.view_id = Some(i64::from(item.item_id));
        result.title = item.title;
        Ok(result)
    }

    /// Run an unsaved definition, as `run` does for a saved one; the builder's preview uses it.
    pub fn run_definition(
        &self,
        definition: &Value,
        limit: Option<i64>,
        offset: Option<i64>,
        account: &Account,
    ) -> Result<ViewResultDto, ViewServiceError> {
        let parsed = saved_view::parse_definition_value(definition)?;
        self.execute(parsed, limit, offset, account)
    }

    /// Execute a parsed definition for one page, grouping it when the definition says so.
    fn execute(
        &self,
        parsed: ParsedView,
        limit: Option<i64>,
        offset: Option<i64>,
        account: &Account,
    ) -> Result<ViewResultDto, ViewServiceError> {
        let ParsedView {
            definition,
            warnings,
        } = parsed;
        let mut query = definition.query.clone();
        query.limit = Some(
            limit
                .or(definition.query.limit)
                .unwrap_or(DEFAULT_LIMIT)
                .clamp(1, MAX_LIMIT),
        );
        query.offset = offset.unwrap_or(definition.query.offset).max(0);
        let (items, total) = self.item_service.query_items(&query, account)?;
        let groups = match &definition.group_by {
            Some(key) => self.group(&items, key, account),
            None => Vec::new(),
        };
        Ok(ViewResultDto {
            view_id: None,
            title: String::from(UNSAVED_TITLE),
            total,
            offset: query.offset,
            items: items.iter().map(ItemDto::from).collect(),
            group_by: definition.group_by,
            groups,
            columns: definition.columns,
            display: definition.display,
            warnings,
        })
    }

    /// Explain how the view `view_id` evaluates today: the resolved query, total and timing.
    pub fn inspect(
        &self,
        view_id: &Id,
        account: &Account,
    ) -> Result<ViewInspectDto, ViewServiceError> {
        let (item, parsed) = self.load(view_id, account)?;
        let mut report = self.evaluate(parsed, account)?;
        report.view_id = Some(i64::from(item.item_id));
        report.title = Some(item.title);
        Ok(report)
    }

    /// Check an unsaved definition: parse it, run its query once, and report like `inspect`.
    pub fn validate(
        &self,
        definition: &Value,
        account: &Account,
    ) -> Result<ViewInspectDto, ViewServiceError> {
        let parsed = saved_view::parse_definition_value(definition)?;
        self.evaluate(parsed, account)
    }

    /// Load a Saved View item and parse its definition.
    fn load(
        &self,
        view_id: &Id,
        account: &Account,
    ) -> Result<(Item, ParsedView), ViewServiceError> {
        let item = self
            .item_service
            .get_item_by_id(view_id, account)?
            .ok_or(ViewServiceError::NotFound)?;
        if !item.types.iter().any(|t| t.name == SAVED_VIEW_TYPE) {
            return Err(ViewServiceError::Invalid(format!(
                "'{}' is not a {SAVED_VIEW_TYPE}; assign the type and add a ```zealot-view block",
                item.title
            )));
        }
        let parsed = saved_view::parse_content(&item.content)?;
        Ok((item, parsed))
    }

    /// Resolve tokens, execute the query for its total, and time it.
    fn evaluate(
        &self,
        parsed: ParsedView,
        account: &Account,
    ) -> Result<ViewInspectDto, ViewServiceError> {
        let tz = clock::timezone_for(account);
        let today = clock::local_today(account);
        let resolved_query = query_tokens::resolve_query(&parsed.definition.query, today)
            .map_err(ViewServiceError::Invalid)?;
        let mut probe = parsed.definition.query.clone();
        probe.limit = Some(1);
        let started = Instant::now();
        let (_, total) = self.item_service.query_items(&probe, account)?;
        Ok(ViewInspectDto {
            view_id: None,
            title: None,
            definition: parsed.definition,
            resolved_query,
            today: today.format("%Y-%m-%d").to_string(),
            time_zone: tz.name().to_string(),
            total,
            elapsed_ms: started.elapsed().as_millis() as u64,
            warnings: parsed.warnings,
        })
    }

    /// Group a page of items by attribute `key`, in first-seen order. A list attribute puts the
    /// item in one group per value; items without the attribute share a `null` group.
    fn group(&self, items: &[Item], key: &str, account: &Account) -> Vec<ViewGroupDto> {
        let mut groups: Vec<ViewGroupDto> = Vec::new();
        let mut index: HashMap<String, usize> = HashMap::new();
        let mut titles: HashMap<Id, String> = HashMap::new();
        for item in items {
            let scalars: Vec<Option<&AttributeScalar>> = match item.attributes.get(key) {
                Some(Attribute::Scalar(scalar)) => vec![Some(scalar)],
                Some(Attribute::List(values)) if !values.is_empty() => {
                    values.iter().map(Some).collect()
                }
                _ => vec![None],
            };
            for scalar in scalars {
                let key_value = scalar
                    .and_then(|s| serde_json::to_value(Attribute::Scalar(s.clone())).ok())
                    .unwrap_or(Value::Null);
                let slot = *index.entry(key_value.to_string()).or_insert_with(|| {
                    groups.push(ViewGroupDto {
                        label: self.label(scalar, &mut titles, account),
                        key: key_value.clone(),
                        item_ids: Vec::new(),
                    });
                    groups.len() - 1
                });
                groups[slot].item_ids.push(i64::from(item.item_id));
            }
        }
        groups
    }

    /// Readable group label: the title for item references, the plain value otherwise.
    fn label(
        &self,
        scalar: Option<&AttributeScalar>,
        titles: &mut HashMap<Id, String>,
        account: &Account,
    ) -> String {
        match scalar {
            None => NONE_LABEL.to_string(),
            Some(AttributeScalar::Item(id)) => titles
                .entry(*id)
                .or_insert_with(|| {
                    self.item_service
                        .get_item_by_id(id, account)
                        .ok()
                        .flatten()
                        .map(|item| item.title)
                        .unwrap_or_else(|| format!("#{}", i64::from(*id)))
                })
                .clone(),
            Some(other) => match serde_json::to_value(Attribute::Scalar(other.clone())) {
                Ok(Value::String(text)) => text,
                Ok(value) => value.to_string(),
                Err(_) => String::from("?"),
            },
        }
    }
}
