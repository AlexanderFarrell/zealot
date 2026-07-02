use zealot_domain::item_type::{
    AddItemTypeDto, ItemTypeDto, ItemTypeSummaryDto, UpdateItemTypeDto,
};

use crate::{ApiError, ZealotClient, api::seg};

impl ZealotClient {
    pub async fn list_item_types(&self) -> Result<Vec<ItemTypeDto>, ApiError> {
        self.get("/item_type/").await
    }

    pub async fn item_type_summaries(&self) -> Result<Vec<ItemTypeSummaryDto>, ApiError> {
        self.get("/item_type/summary").await
    }

    pub async fn get_item_type_by_name(&self, name: &str) -> Result<ItemTypeDto, ApiError> {
        self.get(&format!("/item_type/name/{}", seg(name))).await
    }

    pub async fn add_item_type(&self, dto: &AddItemTypeDto) -> Result<ItemTypeDto, ApiError> {
        self.post("/item_type/", dto).await
    }

    pub async fn update_item_type(&self, dto: &UpdateItemTypeDto) -> Result<ItemTypeDto, ApiError> {
        self.patch(&format!("/item_type/{}", dto.type_id), dto).await
    }

    pub async fn delete_item_type(&self, type_id: i64, force: bool) -> Result<(), ApiError> {
        self.delete(&format!("/item_type/{type_id}?force={force}"))
            .await
    }
}
