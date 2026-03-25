use std::sync::Arc;

use crate::repos::{item::ItemRepo};



#[derive(Debug, Clone)]
pub struct AnalysisService {
    item_repo: Arc<dyn ItemRepo>,
}

impl AnalysisService {
    pub fn new(item_repo: &Arc<dyn ItemRepo>) -> Self {
        Self { item_repo: item_repo.clone() }
    }    
}