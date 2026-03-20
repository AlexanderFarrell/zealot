use serde::{Deserialize, Serialize};




pub struct ItemType {

}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemTypeDto {

}

#[derive(Debug, thiserror::Error)]
pub enum ItemTypeError {

}