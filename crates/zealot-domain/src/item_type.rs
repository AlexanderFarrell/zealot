use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{attribute::Attribute, common::id::Id};

#[derive(Debug, Clone)]
pub struct ItemType {
    pub type_id: Id,
    pub is_system: bool,
    pub name: String,
    pub description: String,
    pub required_attributes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ItemTypeRef {
    pub type_id: Id,
    pub is_system: bool,
    pub name: String,
}

// Send DTO

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTypeDto {
    pub type_id: i64,
    pub is_system: bool,
    pub name: String,
    pub description: String,
    pub required_attributes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTypeRefDto {
    pub type_id: i64,
    pub is_system: bool,
    pub name: String,
}

// Receive DTOs

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddItemTypeDto {
    pub name: String,
    pub description: String,
    pub required_attributes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateItemTypeDto {
    pub type_id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub required_attributes: Option<Vec<String>>,
}

// Errors

#[derive(Debug, thiserror::Error)]
pub enum ItemTypeError {}

// Impls

impl ItemType {
    pub fn is_valid(&self, attributes: &HashMap<String, Attribute>) -> bool {
        for key in &self.required_attributes {
            if !attributes.contains_key(key) {
                return false;
            }
        }
        return true;
    }
}

impl From<&ItemType> for ItemTypeRef {
    fn from(value: &ItemType) -> Self {
        Self {
            type_id: value.type_id,
            is_system: value.is_system,
            name: value.name.clone(),
        }
    }
}

impl From<&ItemType> for ItemTypeDto {
    fn from(value: &ItemType) -> Self {
        Self {
            type_id: value.type_id.into(),
            is_system: value.is_system,
            name: value.name.clone(),
            description: value.description.clone(),
            required_attributes: value.required_attributes.clone(),
        }
    }
}

impl From<&ItemTypeRef> for ItemTypeRefDto {
    fn from(value: &ItemTypeRef) -> Self {
        Self {
            type_id: value.type_id.into(),
            is_system: value.is_system,
            name: value.name.clone(),
        }
    }
}
