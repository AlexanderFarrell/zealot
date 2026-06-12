use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use regex::Regex;
use serde_json::Value;
use zealot_domain::{
    account::Account,
    attribute::{
        Attribute, AttributeBaseScalarType, AttributeBaseType, AttributeError, AttributeFilter,
        AttributeFilterDto, AttributeListMode, AttributeScalar,
    },
    common::id::Id,
    item::{
        AddItemCoreDto, AddItemDto, Item, ItemCore, ItemLink, ItemLinkDto, SearchScope,
        UpdateItemCoreDto, UpdateItemDto, relationship,
    },
    item_type::{ItemType, ItemTypeRef},
};

use crate::{
    ports::events::{EventPort, ZealotEvent},
    repos::{
        attribute::AttributeRepo, common::RepoError, item::ItemRepo,
        item_attribute_value::ItemAttributeValueRepo, item_external_link::ItemExternalLinkRepo,
        item_heading::ItemHeadingRepo, item_link::ItemLinkRepo, item_type::ItemTypeRepo,
    },
};

#[derive(Debug, Clone)]
pub struct ItemService {
    item_repo: Arc<dyn ItemRepo>,
    item_attribute_value_repo: Arc<dyn ItemAttributeValueRepo>,
    item_external_link_repo: Arc<dyn ItemExternalLinkRepo>,
    item_heading_repo: Arc<dyn ItemHeadingRepo>,
    item_link_repo: Arc<dyn ItemLinkRepo>,
    item_type_repo: Arc<dyn ItemTypeRepo>,
    attribute_repo: Arc<dyn AttributeRepo>,
    event_port: Arc<dyn EventPort>,
}

#[derive(Debug, thiserror::Error)]
pub enum ItemServiceError {
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("attribute error: {0}")]
    Attribute(#[from] AttributeError),
    #[error("repo error: {0}")]
    Repo(#[from] RepoError),
    #[error("invalid filter: {0}")]
    InvalidFilter(String),
    #[error("invalid id: {0}")]
    InvalidId(String),
    #[error("invalid regex: {0}")]
    InvalidRegex(String),
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub item: Item,
    pub match_scope: SearchScope,
    pub snippet: Option<String>,
}

impl ItemService {
    pub fn new(
        item_repo: &Arc<dyn ItemRepo>,
        item_attribute_value_repo: &Arc<dyn ItemAttributeValueRepo>,
        item_external_link_repo: &Arc<dyn ItemExternalLinkRepo>,
        item_heading_repo: &Arc<dyn ItemHeadingRepo>,
        item_link_repo: &Arc<dyn ItemLinkRepo>,
        item_type_repo: &Arc<dyn ItemTypeRepo>,
        attribute_repo: &Arc<dyn AttributeRepo>,
        event_port: &Arc<dyn EventPort>,
    ) -> Self {
        Self {
            item_repo: item_repo.clone(),
            item_attribute_value_repo: item_attribute_value_repo.clone(),
            item_external_link_repo: item_external_link_repo.clone(),
            item_heading_repo: item_heading_repo.clone(),
            item_link_repo: item_link_repo.clone(),
            item_type_repo: item_type_repo.clone(),
            attribute_repo: attribute_repo.clone(),
            event_port: event_port.clone(),
        }
    }

    // --- Queries ---

    pub fn get_item_by_id(
        &self,
        item_id: &Id,
        account: &Account,
    ) -> Result<Option<Item>, ItemServiceError> {
        match self
            .item_repo
            .get_item_by_id(item_id, account)
            .map_err(ItemServiceError::Repo)?
        {
            Some(item) => Ok(self
                .hydrate_items(vec![item], &account.account_id)?
                .into_iter()
                .next()),
            None => Ok(None),
        }
    }

    pub fn get_items_by_title(
        &self,
        title: &str,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        let items = self
            .item_repo
            .get_items_by_title(title, account)
            .map_err(ItemServiceError::Repo)?;
        self.hydrate_items(items, &account.account_id)
    }

    pub fn get_items_by_ids(
        &self,
        item_ids: &Vec<Id>,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        self.hydrate_item_ids(item_ids, account)
    }

    pub fn search_items_by_title(
        &self,
        term: &str,
        limit: i64,
        offset: i64,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        let items = self
            .item_repo
            .search_items_by_title(term, limit, offset, account)
            .map_err(ItemServiceError::Repo)?;
        self.hydrate_items(items, &account.account_id)
    }

    pub fn search_items(
        &self,
        term: &str,
        scope: SearchScope,
        use_regex: bool,
        limit: i64,
        offset: i64,
        account: &Account,
    ) -> Result<Vec<SearchResult>, ItemServiceError> {
        let compiled_regex = if use_regex {
            let re = Regex::new(&format!("(?i){}", term))
                .map_err(|e| ItemServiceError::InvalidRegex(e.to_string()))?;
            Some(re)
        } else {
            None
        };

        match scope {
            SearchScope::Title => {
                let fetch_limit = if use_regex { 500 } else { limit };
                let fetch_offset = if use_regex { 0 } else { offset };
                let items = self
                    .item_repo
                    .search_items_by_title(term, fetch_limit, fetch_offset, account)
                    .map_err(ItemServiceError::Repo)?;
                let filtered = if let Some(re) = &compiled_regex {
                    items
                        .into_iter()
                        .filter(|i| re.is_match(&i.title))
                        .collect()
                } else {
                    items
                };
                let paged = if use_regex {
                    filtered
                        .into_iter()
                        .skip(offset as usize)
                        .take(limit as usize)
                        .collect()
                } else {
                    filtered
                };
                let hydrated = self.hydrate_items(paged, &account.account_id)?;
                Ok(hydrated
                    .into_iter()
                    .map(|item| SearchResult {
                        item,
                        match_scope: SearchScope::Title,
                        snippet: None,
                    })
                    .collect())
            }
            SearchScope::Content => {
                let fetch_limit = if use_regex { 500 } else { limit };
                let fetch_offset = if use_regex { 0 } else { offset };
                let items = self
                    .item_repo
                    .search_items_by_content(term, fetch_limit, fetch_offset, account)
                    .map_err(ItemServiceError::Repo)?;
                let filtered: Vec<ItemCore> = if let Some(re) = &compiled_regex {
                    items
                        .into_iter()
                        .filter(|i| re.is_match(&i.content))
                        .collect()
                } else {
                    items
                };
                let paged: Vec<ItemCore> = if use_regex {
                    filtered
                        .into_iter()
                        .skip(offset as usize)
                        .take(limit as usize)
                        .collect()
                } else {
                    filtered
                };
                let snippets: Vec<Option<String>> = paged
                    .iter()
                    .map(|i| extract_snippet(&i.content, term, compiled_regex.as_ref()))
                    .collect();
                let hydrated = self.hydrate_items(paged, &account.account_id)?;
                Ok(hydrated
                    .into_iter()
                    .zip(snippets)
                    .map(|(item, snippet)| SearchResult {
                        item,
                        match_scope: SearchScope::Content,
                        snippet,
                    })
                    .collect())
            }
            SearchScope::Heading => {
                let fetch_limit = if use_regex { 500 } else { limit };
                let fetch_offset = if use_regex { 0 } else { offset };
                let pairs = self
                    .item_repo
                    .search_items_by_heading(term, fetch_limit, fetch_offset, account)
                    .map_err(ItemServiceError::Repo)?;
                let filtered: Vec<(ItemCore, String)> = if let Some(re) = &compiled_regex {
                    pairs.into_iter().filter(|(_, h)| re.is_match(h)).collect()
                } else {
                    pairs
                };
                let paged: Vec<(ItemCore, String)> = if use_regex {
                    filtered
                        .into_iter()
                        .skip(offset as usize)
                        .take(limit as usize)
                        .collect()
                } else {
                    filtered
                };
                let heading_texts: Vec<String> = paged.iter().map(|(_, h)| h.clone()).collect();
                let cores: Vec<ItemCore> = paged.into_iter().map(|(c, _)| c).collect();
                let hydrated = self.hydrate_items(cores, &account.account_id)?;
                Ok(hydrated
                    .into_iter()
                    .zip(heading_texts)
                    .map(|(item, heading)| SearchResult {
                        item,
                        match_scope: SearchScope::Heading,
                        snippet: Some(heading),
                    })
                    .collect())
            }
        }
    }

    pub fn get_recent_items(
        &self,
        limit: i64,
        offset: i64,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        let items = self
            .item_repo
            .get_recent_items(limit, offset, account)
            .map_err(ItemServiceError::Repo)?;
        self.hydrate_items(items, &account.account_id)
    }

    pub fn get_random_items(&self, count: usize, account: &Account) -> Result<Vec<Item>, ItemServiceError> {
        use rand::seq::SliceRandom;
        let mut ids = self
            .item_repo
            .get_all_item_ids_for_user(&account.account_id)
            .map_err(ItemServiceError::Repo)?;
        let mut rng = rand::thread_rng();
        ids.shuffle(&mut rng);
        ids.truncate(count);
        self.hydrate_item_ids(&ids, account)
    }

    /// Returns items where the "Root" attribute is boolean `true`.
    pub fn get_root_items(&self, account: &Account) -> Result<Vec<Item>, ItemServiceError> {
        let ids = self
            .item_attribute_value_repo
            .find_item_ids_by_filters(
                &vec![AttributeFilter {
                    key: String::from("Root"),
                    op: zealot_domain::attribute::AttributeFilterOp::Equal,
                    value: Value::Bool(true),
                    list_mode: AttributeListMode::Any,
                }],
                &account.account_id,
                None,
                0,
            )
            .map_err(ItemServiceError::Repo)?;
        self.hydrate_item_ids(&ids, account)
    }

    pub fn get_items_by_type(
        &self,
        type_name: &str,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        let ids = self
            .item_type_repo
            .get_item_ids_for_type_name(type_name, &account.account_id)
            .map_err(ItemServiceError::Repo)?;
        self.hydrate_item_ids(&ids, account)
    }

    pub fn get_children(
        &self,
        item_id: &Id,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        let ids = self
            .item_link_repo
            .get_source_item_ids(item_id, relationship::PARENT, &account.account_id)
            .map_err(ItemServiceError::Repo)?;
        self.hydrate_item_ids(&ids, account)
    }

    pub fn get_related_items(
        &self,
        item_id: &Id,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        let ids = self
            .item_link_repo
            .get_related_item_ids(item_id, &account.account_id)
            .map_err(ItemServiceError::Repo)?;
        self.hydrate_item_ids(&ids, account)
    }

    pub fn get_backlinks(
        &self,
        item_id: &Id,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        let ids = self
            .item_link_repo
            .get_source_item_ids(item_id, "wikilink", &account.account_id)
            .map_err(ItemServiceError::Repo)?;
        self.hydrate_item_ids(&ids, account)
    }

    pub fn filter_items(
        &self,
        filter_dtos: &Vec<AttributeFilterDto>,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        self.filter_items_with_pagination(filter_dtos, None, 0, account)
    }

    pub fn filter_items_paginated(
        &self,
        filter_dtos: &Vec<AttributeFilterDto>,
        limit: i64,
        offset: i64,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        self.filter_items_with_pagination(filter_dtos, Some(limit), offset, account)
    }

    fn filter_items_with_pagination(
        &self,
        filter_dtos: &Vec<AttributeFilterDto>,
        limit: Option<i64>,
        offset: i64,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        let filters: Vec<AttributeFilter> = filter_dtos
            .iter()
            .map(|dto| AttributeFilter::try_from(dto).map_err(ItemServiceError::InvalidFilter))
            .collect::<Result<Vec<_>, _>>()?;

        let ids = self
            .item_attribute_value_repo
            .find_item_ids_by_filters(&filters, &account.account_id, limit, offset)
            .map_err(ItemServiceError::Repo)?;
        self.hydrate_item_ids(&ids, account)
    }

    // --- Mutations ---

    pub fn add_item(
        &self,
        dto: &AddItemDto,
        account: &Account,
    ) -> Result<Option<Item>, ItemServiceError> {
        if dto.title.trim().is_empty() {
            return Err(ItemServiceError::InvalidFilter(String::from(
                "title is required",
            )));
        }

        let attributes = match &dto.attributes {
            Some(raw) => self.parse_attributes_map(raw, &account.account_id)?,
            None => HashMap::new(),
        };

        let type_names = dto.types.clone().unwrap_or_default();
        let item_types = self.resolve_requested_item_types(&type_names, &account.account_id)?;
        self.ensure_valid_for_types(&item_types, &attributes)?;

        let links = match &dto.links {
            Some(raw) => self.parse_links(raw)?,
            None => Vec::new(),
        };
        self.ensure_links_exist(None, &links, account)?;

        let parsed = AddItemCoreDto {
            title: dto.title.clone(),
            content: dto.content.clone(),
        };

        match self
            .item_repo
            .add_item(&parsed, account)
            .map_err(ItemServiceError::Repo)?
        {
            Some(item) => {
                if !attributes.is_empty() {
                    self.item_attribute_value_repo
                        .replace_item_attributes(&item.item_id, &attributes, account)
                        .map_err(ItemServiceError::Repo)?;
                    self.sync_item_links_from_attributes(&item.item_id, &attributes, account)?;
                }

                if !type_names.is_empty() {
                    self.item_type_repo
                        .assign_item_types(&type_names, &item.item_id, &account.account_id)
                        .map_err(ItemServiceError::Repo)?;
                }

                if !links.is_empty() {
                    self.item_link_repo
                        .replace_links_for_item(&item.item_id, &links, account)
                        .map_err(ItemServiceError::Repo)?;
                }

                self.sync_wiki_links_from_content(&item.item_id, &dto.content, account)?;
                self.sync_headings(&item.item_id, &dto.content)?;
                self.sync_external_links(&item.item_id, &dto.content)?;

                let result = self.get_item_by_id(&item.item_id, account)?;
                if let Some(ref created) = result {
                    tracing::info!(account_id = ?account.account_id, item_id = ?created.item_id, title = %created.title, "item created");
                    self.event_port.emit(ZealotEvent::ItemCreated {
                        account_id: account.account_id,
                        item: created.clone(),
                    });
                }
                Ok(result)
            }
            None => Ok(None),
        }
    }

    pub fn update_item(
        &self,
        item_id: &Id,
        dto: &UpdateItemDto,
        account: &Account,
    ) -> Result<Option<Item>, ItemServiceError> {
        if matches!(dto.title.as_ref(), Some(title) if title.trim().is_empty()) {
            return Err(ItemServiceError::InvalidFilter(String::from(
                "title is required",
            )));
        }

        let current_item = self
            .get_item_by_id(item_id, account)?
            .ok_or(ItemServiceError::NotFound)?;
        let item_types =
            self.resolve_assigned_item_types(&current_item.types, &account.account_id)?;

        let parsed_attributes = match &dto.attributes {
            Some(raw) => Some(self.parse_attributes_map(raw, &account.account_id)?),
            None => None,
        };

        let mut merged_attributes = current_item.attributes.clone();
        if let Some(attributes) = &parsed_attributes {
            for (key, value) in attributes {
                merged_attributes.insert(key.clone(), value.clone());
            }
            self.ensure_valid_for_types(&item_types, &merged_attributes)?;
        }

        let links = match &dto.links {
            Some(raw) => Some(self.parse_links(raw)?),
            None => None,
        };
        if let Some(links) = &links {
            self.ensure_links_exist(Some(*item_id), links, account)?;
        }

        let parsed = UpdateItemCoreDto {
            item_id: *item_id,
            title: dto.title.clone(),
            content: dto.content.clone(),
        };

        let title_changed = dto
            .title
            .as_ref()
            .map(|new_title| new_title != &current_item.title)
            .unwrap_or(false);

        match self
            .item_repo
            .update_item(&parsed, account)
            .map_err(ItemServiceError::Repo)?
        {
            Some(item) => {
                if let Some(attributes) = &parsed_attributes {
                    self.item_attribute_value_repo
                        .replace_item_attributes(&item.item_id, attributes, account)
                        .map_err(ItemServiceError::Repo)?;
                }

                if let Some(links) = &links {
                    self.item_link_repo
                        .replace_links_for_item(&item.item_id, links, account)
                        .map_err(ItemServiceError::Repo)?;
                }

                if let Some(content) = &dto.content {
                    self.sync_wiki_links_from_content(&item.item_id, content, account)?;
                    self.sync_headings(&item.item_id, content)?;
                    self.sync_external_links(&item.item_id, content)?;
                }

                if title_changed {
                    self.rebuild_wiki_links_for_account(account)?;
                }

                let result = self.get_item_by_id(&item.item_id, account)?;
                if let Some(ref updated) = result {
                    tracing::info!(account_id = ?account.account_id, item_id = ?updated.item_id, "item updated");
                    self.event_port.emit(ZealotEvent::ItemUpdated {
                        account_id: account.account_id,
                        item: updated.clone(),
                    });
                }
                Ok(result)
            }
            None => Ok(None),
        }
    }

    pub fn delete_item(&self, item_id: &Id, account: &Account) -> Result<(), ItemServiceError> {
        self.item_repo
            .delete_item(item_id, account)
            .map_err(ItemServiceError::Repo)?;
        tracing::info!(account_id = ?account.account_id, ?item_id, "item deleted");
        self.event_port.emit(ZealotEvent::ItemDeleted {
            account_id: account.account_id,
            item_id: *item_id,
        });
        Ok(())
    }

    pub fn set_attributes(
        &self,
        item_id: &Id,
        raw: &HashMap<String, Value>,
        account: &Account,
    ) -> Result<(), ItemServiceError> {
        if raw.len() > 10 {
            return Err(ItemServiceError::InvalidFilter(String::from(
                "please only update 10 attributes at a time",
            )));
        }
        let current_item = self
            .get_item_by_id(item_id, account)?
            .ok_or(ItemServiceError::NotFound)?;
        let item_types =
            self.resolve_assigned_item_types(&current_item.types, &account.account_id)?;
        let parsed = self.parse_attributes_map(raw, &account.account_id)?;
        let mut merged_attributes = current_item.attributes;
        for (key, value) in &parsed {
            merged_attributes.insert(key.clone(), value.clone());
        }
        self.ensure_valid_for_types(&item_types, &merged_attributes)?;
        self.item_attribute_value_repo
            .replace_item_attributes(item_id, &parsed, account)
            .map_err(ItemServiceError::Repo)?;

        // Sync item-typed attributes into item_item_link so navigation
        // (Children panel, "To Parent") reflects the new relationship values.
        self.sync_item_links_from_attributes(item_id, &parsed, account)?;

        tracing::info!(account_id = ?account.account_id, ?item_id, keys = ?raw.keys().collect::<Vec<_>>(), "item attributes set");

        if let Ok(Some(item)) = self.get_item_by_id(item_id, account) {
            for key in raw.keys() {
                self.event_port.emit(ZealotEvent::AttributeSet {
                    account_id: account.account_id,
                    item: item.clone(),
                    attribute_key: key.clone(),
                });
            }
        }
        Ok(())
    }

    pub fn rename_attribute(
        &self,
        item_id: &Id,
        old_key: &str,
        new_key: &str,
        account: &Account,
    ) -> Result<(), ItemServiceError> {
        if old_key == new_key {
            return Err(ItemServiceError::InvalidFilter(String::from(
                "old_key and new_key must be different",
            )));
        }
        let current_item = self
            .get_item_by_id(item_id, account)?
            .ok_or(ItemServiceError::NotFound)?;
        if !current_item.attributes.contains_key(old_key) {
            return Err(ItemServiceError::InvalidFilter(format!(
                "attribute '{old_key}' does not exist"
            )));
        }
        if current_item.attributes.contains_key(new_key) {
            return Err(ItemServiceError::InvalidFilter(format!(
                "attribute '{new_key}' already exists"
            )));
        }
        let item_types =
            self.resolve_assigned_item_types(&current_item.types, &account.account_id)?;
        let mut merged_attributes = current_item.attributes;
        let attribute = merged_attributes.remove(old_key).ok_or_else(|| {
            ItemServiceError::InvalidFilter(format!("attribute '{old_key}' does not exist"))
        })?;
        merged_attributes.insert(String::from(new_key), attribute);
        self.ensure_valid_for_types(&item_types, &merged_attributes)?;
        self.item_attribute_value_repo
            .rename_item_attribute(item_id, old_key, new_key, account)
            .map_err(ItemServiceError::Repo)?;
        tracing::info!(account_id = ?account.account_id, ?item_id, %old_key, %new_key, "item attribute renamed");
        Ok(())
    }

    pub fn delete_attribute(
        &self,
        item_id: &Id,
        key: &str,
        account: &Account,
    ) -> Result<(), ItemServiceError> {
        let current_item = self
            .get_item_by_id(item_id, account)?
            .ok_or(ItemServiceError::NotFound)?;
        let item_types =
            self.resolve_assigned_item_types(&current_item.types, &account.account_id)?;
        let mut merged_attributes = current_item.attributes;
        merged_attributes.remove(key);
        self.ensure_valid_for_types(&item_types, &merged_attributes)?;
        self.item_attribute_value_repo
            .delete_item_attribute(item_id, key, account)
            .map_err(ItemServiceError::Repo)?;

        // If this attribute kind was item-typed, clear its links.
        let kinds = self
            .attribute_repo
            .get_attribute_kinds_for_user(&account.account_id)
            .map_err(ItemServiceError::Repo)?;
        if let Some(kind) = kinds.get(key) {
            let is_item_typed = matches!(
                &kind.base_type,
                AttributeBaseType::Scalar(AttributeBaseScalarType::Item)
                    | AttributeBaseType::List(AttributeBaseScalarType::Item)
            );
            if is_item_typed {
                self.item_link_repo
                    .replace_links_by_relationship(item_id, &key.to_lowercase(), &[], account)
                    .map_err(ItemServiceError::Repo)?;
            }
        }

        Ok(())
    }

    pub fn assign_type(
        &self,
        type_name: &str,
        item_id: &Id,
        account: &Account,
    ) -> Result<(), ItemServiceError> {
        let current_item = self
            .get_item_by_id(item_id, account)?
            .ok_or(ItemServiceError::NotFound)?;
        let item_type = self
            .item_type_repo
            .get_item_type_by_name(type_name, &account.account_id)
            .map_err(ItemServiceError::Repo)?
            .ok_or_else(|| {
                ItemServiceError::InvalidFilter(format!("unknown item type: {type_name}"))
            })?;
        self.ensure_valid_for_types(&vec![item_type], &current_item.attributes)?;
        self.item_type_repo
            .assign_item_types(&vec![type_name.to_string()], item_id, &account.account_id)
            .map_err(ItemServiceError::Repo)?;
        tracing::info!(account_id = ?account.account_id, ?item_id, %type_name, "item type assigned");
        if let Ok(Some(item)) = self.get_item_by_id(item_id, account) {
            self.event_port.emit(ZealotEvent::TypeAssigned {
                account_id: account.account_id,
                item,
                type_name: type_name.to_string(),
            });
        }
        Ok(())
    }

    pub fn unassign_type(
        &self,
        type_name: &str,
        item_id: &Id,
        account: &Account,
    ) -> Result<(), ItemServiceError> {
        self.item_type_repo
            .unassign_item_types(&vec![type_name.to_string()], item_id, &account.account_id)
            .map_err(ItemServiceError::Repo)?;
        tracing::info!(account_id = ?account.account_id, ?item_id, %type_name, "item type unassigned");
        if let Ok(Some(item)) = self.get_item_by_id(item_id, account) {
            self.event_port.emit(ZealotEvent::TypeUnassigned {
                account_id: account.account_id,
                item,
                type_name: type_name.to_string(),
            });
        }
        Ok(())
    }

    /// Re-syncs `item_item_link` from all item-typed attributes for every item
    /// owned by the account. Returns the number of items processed.
    pub fn rebuild_links_for_account(&self, account: &Account) -> Result<usize, ItemServiceError> {
        let item_ids = self
            .item_repo
            .get_all_item_ids_for_user(&account.account_id)
            .map_err(ItemServiceError::Repo)?;

        let all_attributes = self
            .item_attribute_value_repo
            .get_attributes_for_items(&item_ids, &account.account_id)
            .map_err(ItemServiceError::Repo)?;

        let count = all_attributes.len();
        for (item_id, attrs) in &all_attributes {
            self.sync_item_links_from_attributes(item_id, attrs, account)?;
        }
        Ok(count)
    }

    // --- Private helpers ---

    fn parse_attributes_map(
        &self,
        raw: &HashMap<String, Value>,
        account_id: &Id,
    ) -> Result<HashMap<String, Attribute>, ItemServiceError> {
        let kinds = self
            .attribute_repo
            .get_attribute_kinds_for_user(account_id)
            .map_err(ItemServiceError::Repo)?;

        let mut result = HashMap::new();
        for (key, value) in raw {
            let attr = if let Some(kind) = kinds.get(key.as_str()) {
                Attribute::single_from_json(value, kind).map_err(ItemServiceError::Attribute)?
            } else {
                Attribute::single_from_json_without_kind(value)
                    .map_err(ItemServiceError::Attribute)?
            };
            result.insert(key.clone(), attr);
        }
        Ok(result)
    }

    fn parse_links(&self, raw: &Vec<ItemLinkDto>) -> Result<Vec<ItemLink>, ItemServiceError> {
        raw.iter()
            .map(|link| {
                let other_item_id = Id::try_from(link.other_item_id)
                    .map_err(|err| ItemServiceError::InvalidId(err.to_string()))?;
                Ok(ItemLink {
                    other_item_id,
                    relationship: link.relationship.clone(),
                })
            })
            .collect()
    }

    fn hydrate_items(
        &self,
        item_cores: Vec<ItemCore>,
        account_id: &Id,
    ) -> Result<Vec<Item>, ItemServiceError> {
        if item_cores.is_empty() {
            return Ok(Vec::new());
        }

        let item_ids: Vec<Id> = item_cores.iter().map(|item| item.item_id).collect();
        let mut attributes = self
            .item_attribute_value_repo
            .get_attributes_for_items(&item_ids, account_id)
            .map_err(ItemServiceError::Repo)?;
        let mut type_refs = self
            .item_type_repo
            .get_item_type_refs_for_items(&item_ids, account_id)
            .map_err(ItemServiceError::Repo)?;
        let mut links = self
            .item_link_repo
            .get_links_for_items(&item_ids, account_id)
            .map_err(ItemServiceError::Repo)?;

        Ok(item_cores
            .into_iter()
            .map(|item| Item {
                item_id: item.item_id,
                title: item.title,
                content: item.content,
                attributes: attributes.remove(&item.item_id).unwrap_or_default(),
                types: type_refs.remove(&item.item_id).unwrap_or_default(),
                links: links.remove(&item.item_id).unwrap_or_default(),
            })
            .collect())
    }

    fn hydrate_item_ids(
        &self,
        item_ids: &Vec<Id>,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        if item_ids.is_empty() {
            return Ok(Vec::new());
        }

        let items = self
            .item_repo
            .get_items_by_ids(item_ids, account)
            .map_err(ItemServiceError::Repo)?;
        self.hydrate_items(items, &account.account_id)
    }

    /// Retrieves item types referenced for the given account
    fn resolve_requested_item_types(
        &self,
        type_names: &Vec<String>,
        account_id: &Id,
    ) -> Result<Vec<ItemType>, ItemServiceError> {
        let mut resolved = Vec::new();

        for name in type_names {
            let item_type = self
                .item_type_repo
                .get_item_type_by_name(name, account_id)
                .map_err(ItemServiceError::Repo)?
                .ok_or_else(|| {
                    ItemServiceError::InvalidFilter(format!("unknown item type: {name}"))
                })?;
            resolved.push(item_type);
        }

        Ok(resolved)
    }

    /// Retrieves item types referenced for the given account.
    fn resolve_assigned_item_types(
        &self,
        item_type_refs: &Vec<ItemTypeRef>,
        account_id: &Id,
    ) -> Result<Vec<ItemType>, ItemServiceError> {
        let mut resolved = Vec::new();

        for item_type_ref in item_type_refs {
            if let Some(item_type) = self
                .item_type_repo
                .get_item_type(&item_type_ref.type_id, account_id)
                .map_err(ItemServiceError::Repo)?
            {
                resolved.push(item_type);
            }
        }

        Ok(resolved)
    }

    fn ensure_valid_for_types(
        &self,
        item_types: &Vec<ItemType>,
        attributes: &HashMap<String, Attribute>,
    ) -> Result<(), ItemServiceError> {
        if let Some(item_type) = item_types
            .iter()
            .find(|item_type| !item_type.is_valid(attributes))
        {
            return Err(ItemServiceError::InvalidFilter(format!(
                "item is missing required attributes for type '{}'",
                item_type.name
            )));
        }

        Ok(())
    }

    /// For each attribute in `parsed` whose kind is `item` or `list(item)`,
    /// replace the corresponding rows in `item_item_link` (keyed by
    /// `relationship = attribute_key.to_lowercase()`). Attributes with other
    /// types are ignored.
    fn sync_item_links_from_attributes(
        &self,
        item_id: &Id,
        parsed: &HashMap<String, Attribute>,
        account: &Account,
    ) -> Result<(), ItemServiceError> {
        let kinds = self
            .attribute_repo
            .get_attribute_kinds_for_user(&account.account_id)
            .map_err(ItemServiceError::Repo)?;

        for (key, attr) in parsed {
            let is_item_typed = kinds
                .get(key.as_str())
                .map(|k| {
                    matches!(
                        &k.base_type,
                        AttributeBaseType::Scalar(AttributeBaseScalarType::Item)
                            | AttributeBaseType::List(AttributeBaseScalarType::Item)
                    )
                })
                .unwrap_or(false);

            if !is_item_typed {
                continue;
            }

            let item_ids: Vec<Id> = match attr {
                Attribute::Scalar(AttributeScalar::Item(id)) => vec![*id],
                Attribute::List(scalars) => scalars
                    .iter()
                    .filter_map(|s| {
                        if let AttributeScalar::Item(id) = s {
                            Some(*id)
                        } else {
                            None
                        }
                    })
                    .collect(),
                _ => vec![],
            };

            self.item_link_repo
                .replace_links_by_relationship(item_id, &key.to_lowercase(), &item_ids, account)
                .map_err(ItemServiceError::Repo)?;
        }

        Ok(())
    }

    /// Extracts all `[[Title]]` and `[[type:Title]]` wiki link titles from content.
    fn extract_wiki_link_titles(content: &str) -> Vec<String> {
        let mut titles = Vec::new();
        let mut chars = content.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '[' && chars.peek() == Some(&'[') {
                chars.next();
                let mut inner = String::new();
                loop {
                    match chars.next() {
                        Some(']') if chars.peek() == Some(&']') => {
                            chars.next();
                            break;
                        }
                        Some(ch) => inner.push(ch),
                        None => break,
                    }
                }
                let title = if let Some(pos) = inner.find(':') {
                    inner[pos + 1..].trim().to_string()
                } else {
                    inner.trim().to_string()
                };
                if !title.is_empty() {
                    titles.push(title);
                }
            }
        }
        titles
    }

    fn sync_wiki_links_from_content(
        &self,
        item_id: &Id,
        content: &str,
        account: &Account,
    ) -> Result<(), ItemServiceError> {
        let titles = Self::extract_wiki_link_titles(content);
        let mut resolved_ids: Vec<Id> = Vec::new();
        for title in &titles {
            let items = self
                .item_repo
                .get_items_by_title(title, account)
                .map_err(ItemServiceError::Repo)?;
            for item in items {
                if item.item_id != *item_id {
                    resolved_ids.push(item.item_id);
                }
            }
        }
        self.item_link_repo
            .replace_links_by_relationship(item_id, "wikilink", &resolved_ids, account)
            .map_err(ItemServiceError::Repo)
    }

    pub fn rebuild_wiki_links_for_account(
        &self,
        account: &Account,
    ) -> Result<usize, ItemServiceError> {
        let item_ids = self
            .item_repo
            .get_all_item_ids_for_user(&account.account_id)
            .map_err(ItemServiceError::Repo)?;
        let items = self
            .item_repo
            .get_items_by_ids(&item_ids, account)
            .map_err(ItemServiceError::Repo)?;
        let count = items.len();
        for item in &items {
            self.sync_wiki_links_from_content(&item.item_id, &item.content, account)?;
        }
        Ok(count)
    }

    fn extract_headings(content: &str) -> Vec<(u8, u32, String)> {
        let mut headings = Vec::new();
        let mut ordinal: u32 = 0;
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix('#') {
                let mut level: u8 = 1;
                let mut remaining = rest;
                while let Some(r) = remaining.strip_prefix('#') {
                    level += 1;
                    remaining = r;
                    if level >= 6 {
                        break;
                    }
                }
                if let Some(text) = remaining.strip_prefix(' ') {
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        headings.push((level, ordinal, text));
                        ordinal += 1;
                    }
                }
            }
        }
        headings
    }

    fn extract_external_links(content: &str) -> Vec<String> {
        let mut urls: Vec<String> = Vec::new();
        let mut chars = content.chars().peekable();
        while let Some(c) = chars.next() {
            // Markdown link [text](url)
            if c == '[' {
                let mut _text = String::new();
                loop {
                    match chars.next() {
                        Some(']') | None => break,
                        Some(ch) => _text.push(ch),
                    }
                }
                if chars.peek() == Some(&'(') {
                    chars.next();
                    let mut url = String::new();
                    loop {
                        match chars.next() {
                            Some(')') | None => break,
                            Some(ch) => url.push(ch),
                        }
                    }
                    let url = url.trim().to_string();
                    if (url.starts_with("http://") || url.starts_with("https://"))
                        && !urls.contains(&url)
                    {
                        urls.push(url);
                    }
                }
                continue;
            }
            // Bare https:// or http:// URL
            if c == 'h' {
                let mut candidate = String::from('h');
                for _ in 0..6 {
                    match chars.peek() {
                        Some(&ch) => {
                            candidate.push(ch);
                            chars.next();
                        }
                        None => break,
                    }
                }
                if candidate == "http://" || candidate == "https:/" {
                    // collect one more char for https://
                    if candidate == "https:/" {
                        match chars.peek() {
                            Some(&'/') => {
                                candidate.push('/');
                                chars.next();
                            }
                            _ => {
                                continue;
                            }
                        }
                    }
                    let mut url = candidate;
                    loop {
                        match chars.peek() {
                            Some(&ch)
                                if !ch.is_whitespace() && ch != ')' && ch != '"' && ch != '\'' =>
                            {
                                url.push(ch);
                                chars.next();
                            }
                            _ => break,
                        }
                    }
                    if !urls.contains(&url) {
                        urls.push(url);
                    }
                }
            }
        }
        urls
    }

    fn sync_headings(&self, item_id: &Id, content: &str) -> Result<(), ItemServiceError> {
        let headings = Self::extract_headings(content);
        self.item_heading_repo
            .replace_for_item(item_id, &headings)
            .map_err(ItemServiceError::Repo)
    }

    fn sync_external_links(&self, item_id: &Id, content: &str) -> Result<(), ItemServiceError> {
        let urls = Self::extract_external_links(content);
        self.item_external_link_repo
            .replace_for_item(item_id, &urls)
            .map_err(ItemServiceError::Repo)
    }

    fn ensure_links_exist(
        &self,
        item_id: Option<Id>,
        links: &Vec<ItemLink>,
        account: &Account,
    ) -> Result<(), ItemServiceError> {
        let mut unique_ids: HashSet<Id> = HashSet::new();

        for link in links {
            if Some(link.other_item_id) == item_id {
                return Err(ItemServiceError::InvalidFilter(String::from(
                    "items cannot link to themselves",
                )));
            }
            unique_ids.insert(link.other_item_id);
        }

        if unique_ids.is_empty() {
            return Ok(());
        }

        let requested_ids: Vec<Id> = unique_ids.into_iter().collect();
        let found_items = self
            .item_repo
            .get_items_by_ids(&requested_ids, account)
            .map_err(ItemServiceError::Repo)?;
        let found_ids: HashSet<Id> = found_items.into_iter().map(|item| item.item_id).collect();

        if found_ids.len() != requested_ids.len() {
            return Err(ItemServiceError::InvalidFilter(String::from(
                "one or more linked items do not exist",
            )));
        }

        Ok(())
    }
}

pub fn extract_snippet(text: &str, term: &str, compiled: Option<&Regex>) -> Option<String> {
    let byte_offset = if let Some(re) = compiled {
        re.find(text).map(|m| m.start())
    } else {
        let lower = text.to_lowercase();
        let lower_term = term.to_lowercase();
        lower.find(&lower_term)
    }?;

    let start = text[..byte_offset]
        .char_indices()
        .rev()
        .nth(49)
        .map(|(i, _)| i)
        .unwrap_or(0);
    let end_from = byte_offset + term.len().min(text.len() - byte_offset);
    let end = text[end_from..]
        .char_indices()
        .nth(50)
        .map(|(i, _)| end_from + i)
        .unwrap_or(text.len());

    Some(text[start..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::ItemService;

    #[test]
    fn extract_wiki_link_titles_basic() {
        let titles = ItemService::extract_wiki_link_titles("See [[Foo]] and [[Bar]].");
        assert_eq!(titles, vec!["Foo", "Bar"]);
    }

    #[test]
    fn extract_wiki_link_titles_type_prefix() {
        let titles =
            ItemService::extract_wiki_link_titles("[[Project:My Project]] and [[My Note]]");
        assert_eq!(titles, vec!["My Project", "My Note"]);
    }

    #[test]
    fn extract_wiki_link_titles_empty_content() {
        assert!(ItemService::extract_wiki_link_titles("no links here").is_empty());
    }

    #[test]
    fn extract_headings_levels_and_ordinals() {
        let content = "# Title\n## Section\n### Subsection\nsome text\n## Another";
        let headings = ItemService::extract_headings(content);
        assert_eq!(headings.len(), 4);
        assert_eq!(headings[0], (1, 0, "Title".to_string()));
        assert_eq!(headings[1], (2, 1, "Section".to_string()));
        assert_eq!(headings[2], (3, 2, "Subsection".to_string()));
        assert_eq!(headings[3], (2, 3, "Another".to_string()));
    }

    #[test]
    fn extract_headings_ignores_non_heading_hash() {
        // A line starting with # but no space after is not a heading
        let content = "#tag\n# Real Heading";
        let headings = ItemService::extract_headings(content);
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0], (1, 0, "Real Heading".to_string()));
    }

    #[test]
    fn extract_headings_empty_content() {
        assert!(ItemService::extract_headings("no headings").is_empty());
    }

    #[test]
    fn extract_external_links_markdown() {
        let content = "See [example](https://example.com) for details.";
        let urls = ItemService::extract_external_links(content);
        assert_eq!(urls, vec!["https://example.com"]);
    }

    #[test]
    fn extract_external_links_bare_url() {
        let content = "Visit https://bare.example.org for more.";
        let urls = ItemService::extract_external_links(content);
        assert_eq!(urls, vec!["https://bare.example.org"]);
    }

    #[test]
    fn extract_external_links_mixed_deduplicates() {
        let content = "See [link](https://example.com) and also https://example.com again.";
        let urls = ItemService::extract_external_links(content);
        assert_eq!(urls, vec!["https://example.com"]);
    }

    #[test]
    fn extract_external_links_no_links() {
        assert!(ItemService::extract_external_links("plain text with no URLs").is_empty());
    }

    #[test]
    fn extract_external_links_http_and_https() {
        let content = "http://old.example.com and https://new.example.com";
        let urls = ItemService::extract_external_links(content);
        assert!(urls.contains(&"http://old.example.com".to_string()));
        assert!(urls.contains(&"https://new.example.com".to_string()));
    }

    // ── extract_snippet ───────────────────────────────────────────────────────

    #[test]
    fn extract_snippet_finds_match_in_middle() {
        use super::extract_snippet;
        let content = "Some introductory text. Here is the target keyword right in the middle of a longer passage that continues after.";
        let snippet = extract_snippet(content, "target keyword", None);
        assert!(snippet.is_some());
        let s = snippet.unwrap();
        assert!(
            s.contains("target keyword"),
            "snippet should include the match term"
        );
    }

    #[test]
    fn extract_snippet_returns_none_when_no_match() {
        use super::extract_snippet;
        let snippet = extract_snippet("Nothing interesting here.", "zzzmissing", None);
        assert!(snippet.is_none());
    }

    #[test]
    fn extract_snippet_with_regex_match() {
        use super::extract_snippet;
        use regex::Regex;
        let re = Regex::new("(?i)hello").unwrap();
        let snippet = extract_snippet("Some text HELLO world here.", "hello", Some(&re));
        assert!(snippet.is_some());
    }

    #[test]
    fn extract_snippet_with_regex_no_match() {
        use super::extract_snippet;
        use regex::Regex;
        let re = Regex::new("(?i)zzz").unwrap();
        let snippet = extract_snippet("No match at all.", "zzz", Some(&re));
        assert!(snippet.is_none());
    }

    #[test]
    fn invalid_regex_returns_error() {
        use super::ItemServiceError;
        let result = regex::Regex::new("[unclosed");
        match result {
            Err(e) => {
                let err = ItemServiceError::InvalidRegex(e.to_string());
                assert!(err.to_string().contains("invalid regex"));
            }
            Ok(_) => panic!("expected regex error"),
        }
    }
}
