use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    attribute::{Attribute, AttributeError, AttributeScalar},
    common::{
        id::{Id, IdError},
        strings::StringsError,
    },
    item_type::{ItemTypeError, ItemTypeRef, ItemTypeRefDto},
};

// Domain

#[derive(Debug, Clone)]
pub struct Item {
    pub item_id: Id,
    pub title: String,
    pub content: String,
    pub attributes: HashMap<String, Attribute>,
    pub types: Vec<ItemTypeRef>,
    pub links: Vec<ItemLink>,
}

#[derive(Debug, Clone)]
pub struct ItemCore {
    pub item_id: Id,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemRelationship {
    Parent,
    Blocks,
    Tag,
    Topic,
    Other,
}

#[derive(Debug, Clone)]
pub struct ItemLink {
    pub other_item_id: Id,
    pub relationship: ItemRelationship,
}

// Send DTOs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDto {
    pub item_id: i64,
    pub title: String,
    pub content: String,
    pub attributes: Value,
    pub types: Vec<ItemTypeRefDto>,
    pub links: Vec<ItemLinkDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemLinkDto {
    pub other_item_id: i64,
    pub relationship: ItemRelationship,
}

// Receive DTOs

#[derive(Serialize, Deserialize)]
pub struct AddItemDto {
    pub title: String,
    pub content: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, Value>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<ItemLinkDto>>,
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

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<ItemLinkDto>>,
}

pub struct AddItemCoreDto {
    pub title: String,
    pub content: String,
}

pub struct UpdateItemCoreDto {
    pub item_id: Id,
    pub title: Option<String>,
    pub content: Option<String>,
}

// Errors

#[derive(Debug, thiserror::Error)]
pub enum ItemError {
    #[error("invalid item id: {err:?}")]
    InvalidId { err: IdError },

    #[error("invalid title: {err:?}")]
    InvalidTitle { err: StringsError },

    #[error("invalid content: {err:?}")]
    InvalidContent { err: StringsError },

    #[error("invalid attribute: {err:?}")]
    InvalidAttribute { err: AttributeError },

    #[error("invalid attribute: {err:?}")]
    InvalidItemType { err: ItemTypeError },
}

// Impls

impl Item {
    // // I think we want the repository to make items. We cannot just hydrate the item
    // // without some queries.
    // pub fn from_dto(dto: ItemDto,
    //         kinds: &HashMap<String, AttributeKind>,
    //         types: Vec<Ite>
    //     ) -> Result<Self, ItemError> {
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
                return format!("{} {}", icon, self.title);
            } else {
                return format!("?Unknown Icon? {}", self.title);
            }
        }

        self.title.clone()
    }

    pub fn linked_item_ids(&self, relationship: ItemRelationship) -> Vec<Id> {
        self.links
            .iter()
            .filter(|link| link.relationship == relationship)
            .map(|link| link.other_item_id)
            .collect()
    }

    pub fn parent_ids(&self) -> Vec<Id> {
        self.linked_item_ids(ItemRelationship::Parent)
    }
}

impl From<&ItemLink> for ItemLinkDto {
    fn from(value: &ItemLink) -> Self {
        Self {
            other_item_id: value.other_item_id.into(),
            relationship: value.relationship,
        }
    }
}

impl From<&Item> for ItemDto {
    fn from(value: &Item) -> ItemDto {
        ItemDto {
            item_id: value.item_id.into(),
            title: value.title.clone(),
            content: value.content.clone(),
            attributes: serde_json::to_value(&value.attributes).unwrap(), // Serde JSON serialization failure is not expected.
            types: value
                .types
                .iter()
                .map(ItemTypeRefDto::from)
                .collect(),
            links: value
                .links
                .iter()
                .map(ItemLinkDto::from)
                .collect(),
        }
    }
}
