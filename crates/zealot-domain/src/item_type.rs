use serde::{Deserialize, Serialize};




#[derive(Debug, Clone)]
pub struct ItemType {

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTypeDto {

}

#[derive(Debug, thiserror::Error)]
pub enum ItemTypeError {

}