use std::{collections::HashMap, sync::Arc};

use serde_json::Value;
use zealot_domain::{
    account::Account,
    attribute::{Attribute, AttributeError, AttributeFilter, AttributeFilterDto, AttributeListMode, AttributeScalar},
    common::id::Id,
    item::{AddItemDto, AddItemParsedDto, Item, UpdateItemDto, UpdateItemParsedDto},
};

use crate::repos::{attribute::AttributeRepo, common::RepoError, item::ItemRepo};

#[derive(Debug, Clone)]
pub struct ItemService {
    item_repo: Arc<dyn ItemRepo>,
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
    pub fn new(item_repo: &Arc<dyn ItemRepo>, attribute_repo: &Arc<dyn AttributeRepo>) -> Self {
        Self {
            item_repo: item_repo.clone(),
            attribute_repo: attribute_repo.clone(),
        }
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

    // --- Queries ---

    pub fn get_item_by_id(
        &self,
        item_id: &Id,
        account: &Account,
    ) -> Result<Option<Item>, ItemServiceError> {
        self.item_repo.get_item_by_id(item_id, account).map_err(ItemServiceError::Repo)
    }

    pub fn get_items_by_title(
        &self,
        title: &str,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        self.item_repo.get_items_by_title(title, account).map_err(ItemServiceError::Repo)
    }

    pub fn search_items_by_title(
        &self,
        term: &str,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        self.item_repo.search_items_by_title(term, account).map_err(ItemServiceError::Repo)
    }

    /// Returns items where the "Root" attribute is boolean `true`.
    pub fn get_root_items(&self, account: &Account) -> Result<Vec<Item>, ItemServiceError> {
        let items = self
            .item_repo
            .get_items_containing_attribute("Root", account)
            .map_err(ItemServiceError::Repo)?;

        Ok(items
            .into_iter()
            .filter(|item| {
                matches!(
                    item.attributes.get("Root"),
                    Some(Attribute::Scalar(AttributeScalar::Boolean(true)))
                )
            })
            .collect())
    }

    pub fn get_items_by_type(
        &self,
        type_name: &str,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        self.item_repo.get_items_by_type(type_name, account).map_err(ItemServiceError::Repo)
    }

    /// Returns items whose Parent attribute contains this item's title.
    pub fn get_children(
        &self,
        item_id: &Id,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        let item = self
            .item_repo
            .get_item_by_id(item_id, account)
            .map_err(ItemServiceError::Repo)?
            .ok_or(ItemServiceError::NotFound)?;

        let filter = AttributeFilter {
            key: String::from("Parent"),
            op: zealot_domain::attribute::AttributeFilterOp::Equal,
            value: Value::String(item.title.clone()),
            list_mode: AttributeListMode::Any,
        };

        self.item_repo
            .get_items_by_attr_filter(&vec![filter], account)
            .map_err(ItemServiceError::Repo)
    }

    pub fn get_related_items(
        &self,
        item_id: &Id,
        account: &Account,
    ) -> Result<Vec<Item>, ItemServiceError> {
        self.item_repo.get_related_items(item_id, account).map_err(ItemServiceError::Repo)
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

        self.item_repo
            .get_items_by_attr_filter(&filters, account)
            .map_err(ItemServiceError::Repo)
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

        let type_names = dto.types.as_ref().map(|types| {
            types.iter().map(|t| t.name.clone()).collect()
        });

        let parsed = AddItemParsedDto {
            title: dto.title.clone(),
            content: dto.content.clone(),
            attributes,
            types: type_names,
        };

        self.item_repo.add_item(&parsed, account).map_err(ItemServiceError::Repo)
    }

    pub fn update_item(
        &self,
        item_id: &Id,
        dto: &UpdateItemDto,
        account: &Account,
    ) -> Result<Option<Item>, ItemServiceError> {
        let attributes = match &dto.attributes {
            Some(raw) => Some(self.parse_attributes_map(raw, &account.account_id)?),
            None => None,
        };

        let parsed = UpdateItemParsedDto {
            item_id: *item_id,
            title: dto.title.clone(),
            content: dto.content.clone(),
            attributes,
        };

        self.item_repo.update_item(&parsed, account).map_err(ItemServiceError::Repo)
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
        let parsed = self.parse_attributes_map(raw, &account.account_id)?;
        self.item_repo
            .set_item_attributes(item_id, &parsed, account)
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
        self.item_repo
            .rename_item_attribute(item_id, old_key, new_key, account)
            .map_err(ItemServiceError::Repo)
    }

    pub fn delete_attribute(
        &self,
        item_id: &Id,
        key: &str,
        account: &Account,
    ) -> Result<(), ItemServiceError> {
        self.item_repo
            .delete_item_attribute(item_id, key, account)
            .map_err(ItemServiceError::Repo)
    }

    pub fn assign_type(
        &self,
        type_name: &str,
        item_id: &Id,
        account: &Account,
    ) -> Result<(), ItemServiceError> {
        self.item_repo
            .assign_item_types(&vec![type_name.to_string()], item_id, account)
            .map_err(ItemServiceError::Repo)
    }

    pub fn unassign_type(
        &self,
        type_name: &str,
        item_id: &Id,
        account: &Account,
    ) -> Result<(), ItemServiceError> {
        self.item_repo
            .unassign_item_types(&vec![type_name.to_string()], item_id, account)
            .map_err(ItemServiceError::Repo)
    }
}
