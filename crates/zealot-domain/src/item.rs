use std::{any, collections::HashMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{attribute::{Attribute, AttributeError}, common::{id::{Id, IdError}, strings::StringsError}, item_type::{ItemType, ItemTypeDto, ItemTypeError}};

// Domain

pub struct Item {
    pub item_id: Id,
    pub title: String,
    pub content: String,
    pub attributes: HashMap<String, Attribute>,
    pub types: Vec<ItemType>,
    pub related: Vec<Item>,
}

// DTOs

#[derive(Serialize, Deserialize)]
pub struct ItemDto {
    pub item_id: i64,
    pub title: String,
    pub content: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, Value>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<ItemTypeDto>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related: Option<Vec<ItemDto>>,
}

#[derive(Serialize, Deserialize)]
pub struct AddItemDto {
    pub title: String,
    pub content: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, Value>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<ItemTypeDto>>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateItemDto {
    pub item_id: i64,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, Value>>,
}

// Errors

#[derive(Debug, thiserror::Error)]
pub enum ItemError {
    #[error("invalid item id: {err:?}")]
    InvalidId{err: IdError},

    #[error("invalid title: {err:?}")]
    InvalidTitle{err: StringsError},

    #[error("invalid content: {err:?}")]
    InvalidContent{err: StringsError},

    #[error("invalid attribute: {err:?}")]
    InvalidAttribute{err: AttributeError},

    #[error("invalid attribute: {err:?}")]
    InvalidItemType{err: ItemTypeError},
}

// Impls

impl Item {
    pub fn display_title(&self) -> String {
        if let Some(icon) = self.attributes.get("Icon") {
            if let Attribute::Text(icon) = icon {
                return format!("{} {}", icon, self.title)
            } else {
                return format!("?Unknown Icon? {}", self.title)
            }
        }

        self.title.clone()
    }

    pub fn children(&self) -> Vec<&Item> {
        return self.where_this_is_a("Parent");
    }

    pub fn where_this_is_a(&self, relation: &str) -> Vec<&Item> {
        self.related
            .iter()
            .filter(|item| {
                match item.attributes.get(relation) {
                    Some(Attribute::ListText(value)) => value.contains(&self.title),
                    Some(Attribute::ListItem(value)) => {
                        return value
                            .iter()
                            .find(|i| i.item_id == self.item_id)
                            .is_some()
                    }
                    _ => false
                }
            })
            .collect()
    }
}

impl TryFrom<ItemDto> for Item {
    type Error = ItemError;

    fn try_from(dto: ItemDto) -> Result<Self, Self::Error> {
        if dto.title.len() == 0 {
            return Err(ItemError::InvalidTitle { err: StringsError::IsEmpty });
        }

        

        Ok(Self { 
            item_id: Id::try_from(dto.item_id)
                .map_err(|err| ItemError::InvalidId { err })?, 
            title: dto.title, 
            content: dto.content, 
            attributes: , 
            types: (), 
            related: () 
        })
    }
}