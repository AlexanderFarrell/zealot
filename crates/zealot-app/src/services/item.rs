use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use serde_json::Value;
use zealot_domain::{
    account::Account,
    attribute::{Attribute, AttributeError, AttributeFilter, AttributeFilterDto, AttributeListMode},
    common::id::Id,
    item::{
        AddItemCoreDto, AddItemDto, Item, ItemCore, ItemLink, ItemLinkDto, ItemRelationship,
        UpdateItemCoreDto, UpdateItemDto,
    },
    item_type::{ItemType, ItemTypeRef},
};

use crate::repos::{
    attribute::AttributeRepo,
    common::RepoError,
    item::ItemRepo,
    item_attribute_value::ItemAttributeValueRepo,
    item_link::ItemLinkRepo,
    item_type::ItemTypeRepo,
};

#[derive(Debug, Clone)]
pub struct ItemService {
    item_repo: Arc<dyn ItemRepo>,
    item_attribute_value_repo: Arc<dyn ItemAttributeValueRepo>,
    item_link_repo: Arc<dyn ItemLinkRepo>,
    item_type_repo: Arc<dyn ItemTypeRepo>,
    attribute_repo: Arc<dyn AttributeRepo>,
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
}

impl ItemService {
    pub fn new(
        item_repo: &Arc<dyn ItemRepo>,
        item_attribute_value_repo: &Arc<dyn ItemAttributeValueRepo>,
        item_link_repo: &Arc<dyn ItemLinkRepo>,
        item_type_repo: &Arc<dyn ItemTypeRepo>,
        attribute_repo: &Arc<dyn AttributeRepo>,
    ) -> Self {
        Self {
            item_repo: item_repo.clone(),
            item_attribute_value_repo: item_attribute_value_repo.clone(),
            item_link_repo: item_link_repo.clone(),
            item_type_repo: item_type_repo.clone(),
            attribute_repo: attribute_repo.clone(),
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
            Some(item) => Ok(self.hydrate_items(vec![item], &account.account_id)?.into_iter().next()),
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
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        let items = self
            .item_repo
            .search_items_by_title(term, account)
            .map_err(ItemServiceError::Repo)?;
        self.hydrate_items(items, &account.account_id)
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
            .get_source_item_ids(item_id, ItemRelationship::Parent, &account.account_id)
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

    pub fn filter_items(
        &self,
        filter_dtos: &Vec<AttributeFilterDto>,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        let filters: Vec<AttributeFilter> = filter_dtos
            .iter()
            .map(|dto| AttributeFilter::try_from(dto).map_err(ItemServiceError::InvalidFilter))
            .collect::<Result<Vec<_>, _>>()?;

        let ids = self
            .item_attribute_value_repo
            .find_item_ids_by_filters(&filters, &account.account_id)
            .map_err(ItemServiceError::Repo)
            ?;
        self.hydrate_item_ids(&ids, account)
    }

    // --- Mutations ---

    pub fn add_item(
        &self,
        dto: &AddItemDto,
        account: &Account,
    ) -> Result<Option<Item>, ItemServiceError> {
        if dto.title.trim().is_empty() {
            return Err(ItemServiceError::InvalidFilter(String::from("title is required")));
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

                self.get_item_by_id(&item.item_id, account)
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
            return Err(ItemServiceError::InvalidFilter(String::from("title is required")));
        }

        let current_item = self.get_item_by_id(item_id, account)?.ok_or(ItemServiceError::NotFound)?;
        let item_types = self.resolve_assigned_item_types(&current_item.types, &account.account_id)?;

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

                self.get_item_by_id(&item.item_id, account)
            }
            None => Ok(None),
        }
    }

    pub fn delete_item(&self, item_id: &Id, account: &Account) -> Result<(), ItemServiceError> {
        self.item_repo.delete_item(item_id, account).map_err(ItemServiceError::Repo)
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
        let current_item = self.get_item_by_id(item_id, account)?.ok_or(ItemServiceError::NotFound)?;
        let item_types = self.resolve_assigned_item_types(&current_item.types, &account.account_id)?;
        let parsed = self.parse_attributes_map(raw, &account.account_id)?;
        let mut merged_attributes = current_item.attributes;
        for (key, value) in &parsed {
            merged_attributes.insert(key.clone(), value.clone());
        }
        self.ensure_valid_for_types(&item_types, &merged_attributes)?;
        self.item_attribute_value_repo
            .replace_item_attributes(item_id, &parsed, account)
            .map_err(ItemServiceError::Repo)
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
        let current_item = self.get_item_by_id(item_id, account)?.ok_or(ItemServiceError::NotFound)?;
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
        let item_types = self.resolve_assigned_item_types(&current_item.types, &account.account_id)?;
        let mut merged_attributes = current_item.attributes;
        let attribute = merged_attributes
            .remove(old_key)
            .ok_or_else(|| ItemServiceError::InvalidFilter(format!("attribute '{old_key}' does not exist")))?;
        merged_attributes.insert(String::from(new_key), attribute);
        self.ensure_valid_for_types(&item_types, &merged_attributes)?;
        self.item_attribute_value_repo
            .rename_item_attribute(item_id, old_key, new_key, account)
            .map_err(ItemServiceError::Repo)
    }

    pub fn delete_attribute(
        &self,
        item_id: &Id,
        key: &str,
        account: &Account,
    ) -> Result<(), ItemServiceError> {
        let current_item = self.get_item_by_id(item_id, account)?.ok_or(ItemServiceError::NotFound)?;
        let item_types = self.resolve_assigned_item_types(&current_item.types, &account.account_id)?;
        let mut merged_attributes = current_item.attributes;
        merged_attributes.remove(key);
        self.ensure_valid_for_types(&item_types, &merged_attributes)?;
        self.item_attribute_value_repo
            .delete_item_attribute(item_id, key, account)
            .map_err(ItemServiceError::Repo)
    }

    pub fn assign_type(
        &self,
        type_name: &str,
        item_id: &Id,
        account: &Account,
    ) -> Result<(), ItemServiceError> {
        let current_item = self.get_item_by_id(item_id, account)?.ok_or(ItemServiceError::NotFound)?;
        let item_type = self
            .item_type_repo
            .get_item_type_by_name(type_name, &account.account_id)
            .map_err(ItemServiceError::Repo)?
            .ok_or_else(|| ItemServiceError::InvalidFilter(format!("unknown item type: {type_name}")))?;
        self.ensure_valid_for_types(&vec![item_type], &current_item.attributes)?;
        self.item_type_repo
            .assign_item_types(&vec![type_name.to_string()], item_id, &account.account_id)
            .map_err(ItemServiceError::Repo)
    }

    pub fn unassign_type(
        &self,
        type_name: &str,
        item_id: &Id,
        account: &Account,
    ) -> Result<(), ItemServiceError> {
        self.item_type_repo
            .unassign_item_types(&vec![type_name.to_string()], item_id, &account.account_id)
            .map_err(ItemServiceError::Repo)
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
                Attribute::single_from_json_without_kind(value).map_err(ItemServiceError::Attribute)?
            };
            result.insert(key.clone(), attr);
        }
        Ok(result)
    }

    fn parse_links(&self, raw: &Vec<ItemLinkDto>) -> Result<Vec<ItemLink>, ItemServiceError> {
        raw.iter()
            .map(|link| {
                let other_item_id =
                    Id::try_from(link.other_item_id).map_err(|err| ItemServiceError::InvalidId(err.to_string()))?;
                Ok(ItemLink {
                    other_item_id,
                    relationship: link.relationship,
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
                .ok_or_else(|| ItemServiceError::InvalidFilter(format!("unknown item type: {name}")))?;
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
        if let Some(item_type) = item_types.iter().find(|item_type| !item_type.is_valid(attributes)) {
            return Err(ItemServiceError::InvalidFilter(format!(
                "item is missing required attributes for type '{}'",
                item_type.name
            )));
        }

        Ok(())
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
