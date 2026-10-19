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
        AddItemCoreDto, AddItemDto, Item, ItemCore, ItemLink, ItemLinkDto, ItemQueryDto,
        SearchScope, UpdateItemCoreDto, UpdateItemDto, relationship,
    },
    item_content::{
        self, ContentEditDto, ContentError, ContentWriteDto, GrepHitDto, GrepPageDto, GrepQueryDto,
    },
    item_type::{ItemType, ItemTypeRef},
    query_tokens,
};

use crate::{
    clock,
    ports::events::{EventPort, ZealotEvent},
    repos::{
        attribute::AttributeRepo,
        common::RepoError,
        item::ItemRepo,
        item_attribute_value::{ItemAttributeValueRepo, ItemQuery, ItemSort},
        item_external_link::ItemExternalLinkRepo,
        item_heading::ItemHeadingRepo,
        item_link::ItemLinkRepo,
        item_type::ItemTypeRepo,
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
    /// A targeted content read or edit that cannot apply (text not found,
    /// ambiguous heading, and so on). A version conflict is `Conflict`.
    #[error("{0}")]
    Content(ContentError),
    /// The content changed since the caller's `expected_version`.
    #[error("{0}")]
    Conflict(String),
}

impl From<ContentError> for ItemServiceError {
    fn from(err: ContentError) -> Self {
        match err {
            ContentError::VersionConflict { .. } => ItemServiceError::Conflict(err.to_string()),
            other => ItemServiceError::Content(other),
        }
    }
}

/// Default and maximum matching items per `grep_items` page.
const GREP_DEFAULT_LIMIT: i64 = 20;
const GREP_MAX_LIMIT: i64 = 100;
/// Upper bound on items a scoped grep considers.
const GREP_MAX_CANDIDATES: i64 = 100_000;

/// Message for a compare-and-swap that lost to a concurrent write.
/// Items re-resolved per batch when rebuilding or re-linking wikilinks.
const WIKILINK_PAGE: i64 = 500;

const CONCURRENT_EDIT: &str =
    "the item changed while the edit was applied; re-read it and retry the edit";

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
        Ok(self
            .search_items(term, SearchScope::Title, false, limit, offset, account)?
            .into_iter()
            .map(|hit| hit.item)
            .collect())
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
        self.search_items_with_archived(term, scope, use_regex, limit, offset, false, account)
    }

    pub fn search_items_with_archived(
        &self,
        term: &str,
        scope: SearchScope,
        use_regex: bool,
        limit: i64,
        offset: i64,
        include_archived: bool,
        account: &Account,
    ) -> Result<Vec<SearchResult>, ItemServiceError> {
        let compiled_regex = if use_regex {
            Some(
                Regex::new(&format!("(?i){}", term))
                    .map_err(|e| ItemServiceError::InvalidRegex(e.to_string()))?,
            )
        } else {
            None
        };
        scan_regex_matches(
            limit,
            offset,
            |scan_offset| {
                let fetch_term = if use_regex { "" } else { term };
                let (cores, headings): (Vec<ItemCore>, Vec<String>) = match scope {
                    SearchScope::Title => (
                        self.item_repo.search_items_by_title(
                            fetch_term,
                            REGEX_SCAN_BATCH,
                            scan_offset,
                            account,
                        )?,
                        vec![],
                    ),
                    SearchScope::Content => (
                        self.item_repo.search_items_by_content(
                            fetch_term,
                            REGEX_SCAN_BATCH,
                            scan_offset,
                            account,
                        )?,
                        vec![],
                    ),
                    SearchScope::Heading => self
                        .item_repo
                        .search_items_by_heading(
                            fetch_term,
                            REGEX_SCAN_BATCH,
                            scan_offset,
                            account,
                        )?
                        .into_iter()
                        .unzip(),
                };
                let items = self.hydrate_items(cores, &account.account_id)?;
                Ok(items
                    .into_iter()
                    .enumerate()
                    .map(|(n, item)| {
                        let snippet = match scope {
                            SearchScope::Title => None,
                            SearchScope::Content => {
                                extract_snippet(&item.content, term, compiled_regex.as_ref())
                            }
                            SearchScope::Heading => headings.get(n).cloned(),
                        };
                        SearchResult {
                            item,
                            snippet,
                            match_scope: scope.clone(),
                        }
                    })
                    .collect())
            },
            |hit| {
                (include_archived || !hit.item.is_archived())
                    && compiled_regex.as_ref().is_none_or(|re| {
                        re.is_match(match scope {
                            SearchScope::Title => &hit.item.title,
                            SearchScope::Content => &hit.item.content,
                            SearchScope::Heading => hit.snippet.as_deref().unwrap_or(""),
                        })
                    })
            },
        )
    }

    pub fn grep_items(
        &self,
        query: &GrepQueryDto,
        account: &Account,
    ) -> Result<GrepPageDto, ItemServiceError> {
        if query.pattern.is_empty() {
            return Err(ItemServiceError::InvalidFilter(String::from(
                "pattern must not be empty",
            )));
        }
        let pattern = if query.regex {
            query.pattern.clone()
        } else {
            regex::escape(&query.pattern)
        };
        let re = Regex::new(&format!("(?i){pattern}"))
            .map_err(|e| ItemServiceError::InvalidRegex(e.to_string()))?;
        let limit = query
            .limit
            .unwrap_or(GREP_DEFAULT_LIMIT)
            .clamp(1, GREP_MAX_LIMIT) as usize;
        let offset = query.offset.max(0) as usize;

        let mut ids = if query.under.is_some() || !query.types.is_empty() {
            let under = query
                .under
                .map(|id| Id::try_from(id).map_err(|e| ItemServiceError::InvalidId(e.to_string())))
                .transpose()?;
            let scope = ItemQuery {
                types: query.types.clone(),
                all_types: false,
                include_archived: query.include_archived,
                under,
                filters: Vec::new(),
                any_of: Vec::new(),
                linked_from: None,
                sort: None,
                limit: GREP_MAX_CANDIDATES,
                offset: 0,
            };
            self.item_attribute_value_repo
                .query_item_ids(&scope, &account.account_id)
                .map_err(ItemServiceError::Repo)?
                .0
        } else {
            self.item_repo
                .get_all_item_ids_for_user(&account.account_id)
                .map_err(ItemServiceError::Repo)?
        };
        ids.sort_by_key(|id| i64::from(*id));

        // Collect one past the page to know whether another page exists.
        let mut skipped = 0;
        let mut hits = Vec::new();
        'scan: for batch in ids.chunks(REGEX_SCAN_BATCH as usize) {
            let items = self.visible_item_ids(&batch.to_vec(), query.include_archived, account)?;
            for item in items {
                let Some((lines, match_count, truncated)) =
                    item_content::grep_lines(&item.content, query.context_lines, |l| {
                        re.is_match(l)
                    })
                else {
                    continue;
                };
                if skipped < offset {
                    skipped += 1;
                    continue;
                }
                hits.push(GrepHitDto {
                    item_id: item.item_id.into(),
                    title: item.title,
                    match_count,
                    lines,
                    truncated,
                });
                if hits.len() > limit {
                    break 'scan;
                }
            }
        }
        let more = hits.len() > limit;
        hits.truncate(limit);
        Ok(GrepPageDto {
            count: hits.len(),
            next_offset: more.then_some((offset + limit) as i64),
            items: hits,
        })
    }

    pub fn get_recent_items(
        &self,
        limit: i64,
        offset: i64,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        self.get_recent_items_with_archived(limit, offset, false, account)
    }

    pub fn get_recent_items_with_archived(
        &self,
        limit: i64,
        offset: i64,
        include_archived: bool,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        scan_regex_matches(
            limit,
            offset,
            |scan_offset| {
                let cores =
                    self.item_repo
                        .get_recent_items(REGEX_SCAN_BATCH, scan_offset, account)?;
                // Keep the raw batch length for scanning, then test visibility before paging.
                self.hydrate_items(cores, &account.account_id)
            },
            |item| include_archived || !item.is_archived(),
        )
    }

    pub fn get_random_items(
        &self,
        count: usize,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        self.get_random_items_with_archived(count, false, account)
    }

    pub fn get_random_items_with_archived(
        &self,
        count: usize,
        include_archived: bool,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        use rand::seq::SliceRandom;
        let mut ids = self
            .item_repo
            .get_all_item_ids_for_user(&account.account_id)
            .map_err(ItemServiceError::Repo)?;
        let mut rng = rand::thread_rng();
        ids.shuffle(&mut rng);
        ids = self
            .visible_item_ids(&ids, include_archived, account)?
            .iter()
            .map(|i| i.item_id)
            .collect();
        ids.truncate(count);
        self.visible_item_ids(&ids, include_archived, account)
    }

    /// Returns items where the "Root" attribute is boolean `true`.
    pub fn get_root_items(&self, account: &Account) -> Result<Vec<Item>, ItemServiceError> {
        self.get_root_items_with_archived(false, account)
    }

    pub fn get_root_items_with_archived(
        &self,
        include_archived: bool,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
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
        self.visible_item_ids(&ids, include_archived, account)
    }

    pub fn get_items_by_type(
        &self,
        type_name: &str,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        self.get_items_by_type_with_archived(type_name, false, account)
    }

    pub fn get_items_by_type_with_archived(
        &self,
        type_name: &str,
        include_archived: bool,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        let ids = self
            .item_type_repo
            .get_item_ids_for_type_name(type_name, &account.account_id)
            .map_err(ItemServiceError::Repo)?;
        self.visible_item_ids(&ids, include_archived, account)
    }

    pub fn get_children(
        &self,
        item_id: &Id,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        self.get_children_with_archived(item_id, false, account)
    }

    pub fn get_children_with_archived(
        &self,
        item_id: &Id,
        include_archived: bool,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        let ids = self
            .item_link_repo
            .get_source_item_ids(item_id, relationship::PARENT, &account.account_id)
            .map_err(ItemServiceError::Repo)?;
        self.visible_item_ids(&ids, include_archived, account)
    }

    pub fn get_related_items(
        &self,
        item_id: &Id,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        self.get_related_items_with_archived(item_id, false, account)
    }

    pub fn get_related_items_with_archived(
        &self,
        item_id: &Id,
        include_archived: bool,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        let ids = self
            .item_link_repo
            .get_related_item_ids(item_id, &account.account_id)
            .map_err(ItemServiceError::Repo)?;
        self.visible_item_ids(&ids, include_archived, account)
    }

    pub fn get_backlinks(
        &self,
        item_id: &Id,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        self.get_backlinks_with_archived(item_id, false, account)
    }

    pub fn get_backlinks_with_archived(
        &self,
        item_id: &Id,
        include_archived: bool,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        let ids = self
            .item_link_repo
            .get_source_item_ids(item_id, "wikilink", &account.account_id)
            .map_err(ItemServiceError::Repo)?;
        self.visible_item_ids(&ids, include_archived, account)
    }

    pub fn filter_items(
        &self,
        filter_dtos: &Vec<AttributeFilterDto>,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        self.filter_items_with_pagination(filter_dtos, None, 0, false, account)
    }

    pub fn filter_items_paginated(
        &self,
        filter_dtos: &Vec<AttributeFilterDto>,
        limit: i64,
        offset: i64,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        self.filter_items_with_pagination(filter_dtos, Some(limit), offset, false, account)
    }

    /// Query items by type, subtree, attribute filters, and sort; returns one page plus the total.
    pub fn query_items(
        &self,
        dto: &ItemQueryDto,
        account: &Account,
    ) -> Result<(Vec<Item>, i64), ItemServiceError> {
        let today = clock::local_today(account);
        let filters = parse_filters(&dto.filters, today)?;
        let any_of = dto
            .any_of
            .iter()
            .map(|group| {
                if group.is_empty() {
                    return Err(ItemServiceError::InvalidFilter(String::from(
                        "any_of groups must not be empty",
                    )));
                }
                parse_filters(group, today)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let all_types = match dto.type_mode.as_str() {
            "" | "any" => false,
            "all" => true,
            other => {
                return Err(ItemServiceError::InvalidFilter(format!(
                    "type_mode must be 'any' or 'all', got '{other}'"
                )));
            }
        };
        let sort = match &dto.sort {
            None => None,
            Some(sort) => Some(ItemSort {
                key: sort.key.clone(),
                descending: match sort.direction.to_lowercase().as_str() {
                    "" | "asc" => false,
                    "desc" => true,
                    other => {
                        return Err(ItemServiceError::InvalidFilter(format!(
                            "sort direction must be 'asc' or 'desc', got '{other}'"
                        )));
                    }
                },
            }),
        };
        let under = dto
            .under
            .map(|id| Id::try_from(id).map_err(|e| ItemServiceError::InvalidFilter(e.to_string())))
            .transpose()?;
        let linked_from = dto
            .linked_from
            .map(|id| Id::try_from(id).map_err(|e| ItemServiceError::InvalidFilter(e.to_string())))
            .transpose()?;
        if dto.types.is_empty()
            && under.is_none()
            && filters.is_empty()
            && any_of.is_empty()
            && linked_from.is_none()
        {
            return Err(ItemServiceError::InvalidFilter(String::from(
                "query needs at least one of types, under, filters, any_of, or linked_from",
            )));
        }
        let query = ItemQuery {
            types: dto.types.clone(),
            all_types,
            include_archived: dto.include_archived,
            under,
            filters,
            any_of,
            linked_from,
            sort,
            limit: dto.limit.unwrap_or(50).clamp(1, 100),
            offset: dto.offset.max(0),
        };
        let (ids, total) = self
            .item_attribute_value_repo
            .query_item_ids(&query, &account.account_id)
            .map_err(ItemServiceError::Repo)?;
        Ok((self.hydrate_item_ids(&ids, account)?, total))
    }

    pub fn filter_items_with_pagination(
        &self,
        filter_dtos: &Vec<AttributeFilterDto>,
        limit: Option<i64>,
        offset: i64,
        include_archived: bool,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        let filters = parse_filters(filter_dtos, clock::local_today(account))?;

        let query = ItemQuery {
            types: vec![],
            all_types: false,
            under: None,
            filters,
            any_of: Vec::new(),
            linked_from: None,
            sort: None,
            include_archived,
            limit: limit.unwrap_or(i64::MAX),
            offset: offset.max(0),
        };
        let (ids, _) = self
            .item_attribute_value_repo
            .query_item_ids(&query, &account.account_id)?;
        self.hydrate_item_ids(&ids, account)
    }

    /// Visibility affects discovery only, never direct reads or link storage.
    fn visible_item_ids(
        &self,
        ids: &Vec<Id>,
        include_archived: bool,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        Ok(self
            .hydrate_item_ids(ids, account)?
            .into_iter()
            .filter(|i| include_archived || !i.is_archived())
            .collect())
    }

    /// Toggle an item's archive flag, optionally with all account-owned descendants.
    pub fn set_archived(
        &self,
        root: &Id,
        archived: bool,
        subtree: bool,
        account: &Account,
    ) -> Result<Vec<Id>, ItemServiceError> {
        let ids = self
            .item_attribute_value_repo
            .set_archived(root, archived, subtree, account)?;
        if ids.is_empty() {
            return Err(ItemServiceError::NotFound);
        }
        Ok(ids)
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
        self.ensure_valid_for_types(&item_types, None, &attributes)?;

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
        item_content::check_version(&current_item.content, dto.expected_version.as_deref())?;
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
            self.ensure_valid_for_types(
                &item_types,
                Some(&current_item.attributes),
                &merged_attributes,
            )?;
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

        // With an expected version, the content write is a compare-and-swap
        // against the body just checked, so a concurrent edit is refused.
        let updated = match (&dto.expected_version, &dto.content) {
            (Some(_), Some(content)) => {
                let swapped = self
                    .item_repo
                    .update_content_if_unchanged(
                        item_id,
                        &current_item.content,
                        dto.title.as_deref(),
                        content,
                        account,
                    )
                    .map_err(ItemServiceError::Repo)?;
                if !swapped {
                    return Err(ItemServiceError::Conflict(CONCURRENT_EDIT.to_string()));
                }
                self.item_repo
                    .get_item_by_id(item_id, account)
                    .map_err(ItemServiceError::Repo)?
            }
            _ => self
                .item_repo
                .update_item(&parsed, account)
                .map_err(ItemServiceError::Repo)?,
        };

        match updated {
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
                    self.relink_after_rename(&current_item.title, &item.title, account)?;
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

    /// Apply one targeted content edit (exact replace, append, or section
    /// rewrite) on the server. The new body is computed from the current one
    /// and written with a compare-and-swap, so nothing between the read and
    /// the write is lost; `expected_version` additionally refuses the edit if
    /// the body changed since the caller read it.
    pub fn edit_content(
        &self,
        item_id: &Id,
        dto: &ContentEditDto,
        account: &Account,
    ) -> Result<ContentWriteDto, ItemServiceError> {
        let current = self
            .item_repo
            .get_item_by_id(item_id, account)
            .map_err(ItemServiceError::Repo)?
            .ok_or(ItemServiceError::NotFound)?;
        item_content::check_version(&current.content, dto.expected_version.as_deref())?;
        let content = item_content::apply_edit(&current.content, &dto.edit)?;
        let changed_lines = item_content::changed_lines(&current.content, &content);

        if changed_lines.is_some() {
            let swapped = self
                .item_repo
                .update_content_if_unchanged(item_id, &current.content, None, &content, account)
                .map_err(ItemServiceError::Repo)?;
            if !swapped {
                return Err(ItemServiceError::Conflict(CONCURRENT_EDIT.to_string()));
            }
            self.sync_wiki_links_from_content(item_id, &content, account)?;
            self.sync_headings(item_id, &content)?;
            self.sync_external_links(item_id, &content)?;
            if let Some(updated) = self.get_item_by_id(item_id, account)? {
                tracing::info!(account_id = ?account.account_id, ?item_id, ?changed_lines, "item content edited");
                self.event_port.emit(ZealotEvent::ItemUpdated {
                    account_id: account.account_id,
                    item: updated,
                });
            }
        }

        Ok(ContentWriteDto {
            item_id: current.item_id.into(),
            title: current.title,
            version: item_content::content_version(&content),
            content_chars: content.chars().count(),
            changed_lines,
        })
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
        let previous_attributes = current_item.attributes.clone();
        let mut merged_attributes = current_item.attributes;
        for (key, value) in &parsed {
            merged_attributes.insert(key.clone(), value.clone());
        }
        self.ensure_valid_for_types(&item_types, Some(&previous_attributes), &merged_attributes)?;
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
                    old_value: previous_attributes.get(key).cloned(),
                    new_value: item.attributes.get(key).cloned(),
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
        let before = current_item.attributes.clone();
        let mut merged_attributes = current_item.attributes;
        let attribute = merged_attributes.remove(old_key).ok_or_else(|| {
            ItemServiceError::InvalidFilter(format!("attribute '{old_key}' does not exist"))
        })?;
        merged_attributes.insert(String::from(new_key), attribute);
        self.ensure_valid_for_types(&item_types, Some(&before), &merged_attributes)?;
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
        let old_value = current_item.attributes.get(key).cloned();
        let before = current_item.attributes.clone();
        let mut merged_attributes = current_item.attributes;
        merged_attributes.remove(key);
        self.ensure_valid_for_types(&item_types, Some(&before), &merged_attributes)?;
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

        tracing::info!(account_id = ?account.account_id, ?item_id, %key, "item attribute deleted");

        if let Ok(Some(item)) = self.get_item_by_id(item_id, account) {
            self.event_port.emit(ZealotEvent::AttributeSet {
                account_id: account.account_id,
                item,
                attribute_key: key.to_string(),
                old_value,
                new_value: None,
            });
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
        self.ensure_valid_for_types(&vec![item_type], None, &current_item.attributes)?;
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
                Attribute::single_from_json(value, kind)
                    .map_err(|e| ItemServiceError::Attribute(e.for_attribute(key, Some(kind))))?
            } else {
                Attribute::single_from_json_without_kind(value)
                    .map_err(|e| ItemServiceError::Attribute(e.for_attribute(key, None)))?
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

    /// Check `attributes` against each type's requirements. With `before` (the item's attributes
    /// ahead of this edit) an edit passes when it adds no newly missing required key, so an item
    /// already out of compliance can still be repaired one attribute at a time. Without `before`
    /// (create, assign type) every requirement must hold.
    fn ensure_valid_for_types(
        &self,
        item_types: &Vec<ItemType>,
        before: Option<&HashMap<String, Attribute>>,
        attributes: &HashMap<String, Attribute>,
    ) -> Result<(), ItemServiceError> {
        for item_type in item_types {
            let mut missing = item_type.missing_required(attributes);
            if let Some(before) = before {
                let already = item_type.missing_required(before);
                missing.retain(|key| !already.contains(key));
            }
            if !missing.is_empty() {
                return Err(ItemServiceError::InvalidFilter(format!(
                    "item is missing required attributes for type '{}': {}",
                    item_type.name,
                    missing.join(", ")
                )));
            }
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
        let mut titles = Self::extract_wiki_link_titles(content);
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
        titles.sort();
        titles.dedup();
        self.item_link_repo
            .replace_wikilinks(item_id, &titles, &resolved_ids, account)
            .map_err(ItemServiceError::Repo)
    }

    /// Full wikilink rebuild, for repair: re-resolves every `[[Title]]` of the
    /// account, a page of items at a time, and records its references so
    /// renames can then be incremental. Returns the number of items processed.
    pub fn rebuild_wiki_links_for_account(
        &self,
        account: &Account,
    ) -> Result<usize, ItemServiceError> {
        let mut count = 0;
        let mut after_id = 0;
        loop {
            let page = self
                .item_repo
                .get_items_page_after(after_id, WIKILINK_PAGE, account)
                .map_err(ItemServiceError::Repo)?;
            let Some(last) = page.last() else {
                break;
            };
            after_id = i64::from(last.item_id);
            for item in &page {
                self.sync_wiki_links_from_content(&item.item_id, &item.content, account)?;
            }
            count += page.len();
        }
        self.item_link_repo
            .mark_wikilink_refs_complete(&account.account_id)
            .map_err(ItemServiceError::Repo)?;
        Ok(count)
    }

    /// After a rename, re-resolves only the wikilinks the new title can change:
    /// those of items that mention the old title (now dangling or pointing at
    /// another item of that title) or the new one (now resolvable). Falls back
    /// to a full rebuild until the account's references are fully recorded.
    fn relink_after_rename(
        &self,
        old_title: &str,
        new_title: &str,
        account: &Account,
    ) -> Result<(), ItemServiceError> {
        if !self
            .item_link_repo
            .wikilink_refs_complete(&account.account_id)
            .map_err(ItemServiceError::Repo)?
        {
            self.rebuild_wiki_links_for_account(account)?;
            return Ok(());
        }
        let sources = self
            .item_link_repo
            .get_wikilink_source_ids(
                &[old_title.to_string(), new_title.to_string()],
                &account.account_id,
            )
            .map_err(ItemServiceError::Repo)?;
        for batch in sources.chunks(WIKILINK_PAGE as usize) {
            let items = self
                .item_repo
                .get_items_by_ids(&batch.to_vec(), account)
                .map_err(ItemServiceError::Repo)?;
            for item in &items {
                self.sync_wiki_links_from_content(&item.item_id, &item.content, account)?;
            }
        }
        Ok(())
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

/// Rows fetched per repository call while scanning for regex matches.
pub const REGEX_SCAN_BATCH: i64 = 500;

/// Pages through every candidate row in `fetch` (called with a scan offset) and returns the
/// `limit` matches after skipping `offset` matches. Stops early once the page is full.
pub fn scan_regex_matches<T>(
    limit: i64,
    offset: i64,
    mut fetch: impl FnMut(i64) -> Result<Vec<T>, ItemServiceError>,
    is_match: impl Fn(&T) -> bool,
) -> Result<Vec<T>, ItemServiceError> {
    let (limit, offset) = (limit.max(0) as usize, offset.max(0) as usize);
    let mut page = Vec::new();
    let mut skipped = 0;
    let mut scan_offset = 0;
    while page.len() < limit {
        let rows = fetch(scan_offset)?;
        let fetched = rows.len() as i64;
        for row in rows {
            if !is_match(&row) {
                continue;
            }
            if skipped < offset {
                skipped += 1;
            } else if page.len() < limit {
                page.push(row);
            }
        }
        if fetched < REGEX_SCAN_BATCH {
            break;
        }
        scan_offset += fetched;
    }
    Ok(page)
}

/// Parse filter DTOs, resolving relative date tokens (`$today`, `$this_week`, ...) against `today`.
fn parse_filters(
    dtos: &[AttributeFilterDto],
    today: chrono::NaiveDate,
) -> Result<Vec<AttributeFilter>, ItemServiceError> {
    dtos.iter()
        .map(|dto| {
            let mut filter =
                AttributeFilter::try_from(dto).map_err(ItemServiceError::InvalidFilter)?;
            filter.value = query_tokens::resolve_value(&filter.value, today)
                .map_err(ItemServiceError::InvalidFilter)?;
            Ok(filter)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{ItemService, ItemServiceError, REGEX_SCAN_BATCH, scan_regex_matches};

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

    // The pattern is invalid on purpose: this test exercises the error path.
    #[allow(clippy::invalid_regex)]
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

    fn regex_of(pattern: &str) -> regex::Regex {
        regex::Regex::new(&format!("(?i){pattern}")).unwrap()
    }

    /// Serves `rows` in `REGEX_SCAN_BATCH` pages, like a repository listing with an empty term.
    fn scan(rows: &[String], pattern: &str, limit: i64, offset: i64) -> (Vec<String>, usize) {
        let re = regex_of(pattern);
        let mut calls = 0;
        let page = scan_regex_matches(
            limit,
            offset,
            |scan_offset| {
                calls += 1;
                Ok(rows
                    .iter()
                    .skip(scan_offset as usize)
                    .take(REGEX_SCAN_BATCH as usize)
                    .cloned()
                    .collect())
            },
            |row| re.is_match(row),
        )
        .unwrap();
        (page, calls)
    }

    fn titles() -> Vec<String> {
        [
            "AIW001 Alpha",
            "Zeal note",
            "AIW002 Beta",
            "Notes on AIW",
            "aiw003 lower",
            "AIW010 Ten",
        ]
        .iter()
        .map(|t| t.to_string())
        .collect()
    }

    #[test]
    fn regex_scan_matches_anchor_without_literal_prefilter() {
        let (page, _) = scan(&titles(), "^AIW", 10, 0);
        assert_eq!(
            page,
            ["AIW001 Alpha", "AIW002 Beta", "aiw003 lower", "AIW010 Ten"]
        );
    }

    #[test]
    fn regex_scan_matches_character_class() {
        let (page, _) = scan(&titles(), "AIW00[0-9]", 10, 0);
        assert_eq!(page, ["AIW001 Alpha", "AIW002 Beta", "aiw003 lower"]);
    }

    #[test]
    fn regex_scan_matches_alternation() {
        let (page, _) = scan(&titles(), "^(Zeal|Notes)", 10, 0);
        assert_eq!(page, ["Zeal note", "Notes on AIW"]);
    }

    #[test]
    fn regex_scan_is_case_insensitive() {
        let (page, _) = scan(&titles(), "^aiw00[13]", 10, 0);
        assert_eq!(page, ["AIW001 Alpha", "aiw003 lower"]);
    }

    #[test]
    fn regex_scan_pages_over_matches_not_rows() {
        let (first, _) = scan(&titles(), "^AIW", 2, 0);
        let (second, _) = scan(&titles(), "^AIW", 2, 2);
        let (past_end, _) = scan(&titles(), "^AIW", 2, 4);
        assert_eq!(first, ["AIW001 Alpha", "AIW002 Beta"]);
        assert_eq!(second, ["aiw003 lower", "AIW010 Ten"]);
        assert!(past_end.is_empty());
    }

    #[test]
    fn regex_scan_is_not_truncated_at_one_batch() {
        let mut rows: Vec<String> = (0..REGEX_SCAN_BATCH * 2 + 7)
            .map(|i| format!("row {i}"))
            .collect();
        rows.push("AIW-late".to_string());
        let (page, calls) = scan(&rows, "^AIW", 10, 0);
        assert_eq!(page, ["AIW-late"]);
        assert_eq!(calls, 3);
    }

    #[test]
    fn regex_scan_stops_once_page_is_full() {
        let rows: Vec<String> = (0..REGEX_SCAN_BATCH * 3)
            .map(|i| format!("AIW{i}"))
            .collect();
        let (page, calls) = scan(&rows, "^AIW", 5, 0);
        assert_eq!(page.len(), 5);
        assert_eq!(calls, 1);
    }

    #[test]
    fn regex_scan_propagates_fetch_errors() {
        let result: Result<Vec<String>, _> = scan_regex_matches(
            5,
            0,
            |_| Err(ItemServiceError::InvalidRegex("boom".to_string())),
            |_: &String| true,
        );
        assert!(result.is_err());
    }
}
