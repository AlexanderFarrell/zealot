use std::{collections::HashMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{attribute::{Attribute, AttributeError, AttributeScalar}, common::{id::{Id, IdError}, strings::StringsError}, item_type::{ItemType, ItemTypeDto, ItemTypeError}};

// Domain

#[derive(Debug, Clone)]
pub struct Item {
    pub item_id: Id,
    pub title: String,
    pub content: String,
    pub attributes: HashMap<String, Attribute>,
    pub types: Vec<ItemType>,
    pub related: Vec<Item>,
}

// DTOs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDto {
    pub item_id: i64,
    pub title: String,
    pub content: String,
    pub attributes: Value,
    pub types: Vec<ItemTypeDto>,
    pub related: Vec<ItemDto>,
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
    // // I think we want the repository to make items. We cannot just hydrate the item
    // // without some queries.
    // pub fn from_dto(dto: ItemDto, kinds: &HashMap<String, AttributeKind>) -> Result<Self, ItemError> {
    //     if dto.title.len() == 0 {
    //         return Err(ItemError::InvalidTitle { err: StringsError::IsEmpty });
    //     }

    //     Ok(Self { 
    //         item_id: Id::try_from(dto.item_id)
    //             .map_err(|err| ItemError::InvalidId { err })?, 
    //         title: dto.title, 
    //         content: dto.content, 
    //         attributes: (match dto.attributes {
    //             Some(json) => {
    //                 Attribute::from_json(&json, kinds)
    //             },
    //             None => {
    //                 Ok(HashMap::new())
    //             }
    //         }).map_err(|err| ItemError::InvalidAttribute { err })?, 
    //         types: (), 
    //         related: () 
    //     })
    // }

    pub fn display_title(&self) -> String {
        if let Some(icon) = self.attributes.get("Icon") {
            if let Attribute::Scalar(AttributeScalar::Text(icon)) = icon {
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
                    Some(Attribute::List(list)) => {
                        for item in list {
                            match item {
                                AttributeScalar::Item(item_id) => {
                                    return *item_id == self.item_id
                                },
                                _ => return false
                            }
                        }
                        return false
                    },
                    Some(Attribute::Scalar(AttributeScalar::Item(item_id))) => {
                        return *item_id == self.item_id
                    },
                    _ => return false,
                }
            })
            .collect()
    }
}

impl From<&Item> for ItemDto {
    fn from(value: &Item) -> Result<Self, String> {
        Ok(ItemDto {
            item_id: value.item_id.into(),
            title: value.title.clone(),
            content: value.content.clone(),
            attributes: serde_json::to_value(&value.attributes)
                .map_err(|e| Err(e.to_string()))?,
            types: value.types.clone(),
            related: value.related.clone(),
        })
    }
}