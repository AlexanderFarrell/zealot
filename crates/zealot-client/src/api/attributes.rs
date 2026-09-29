use zealot_domain::attribute::{AddAttributeKindDto, UpdateAttributeKindDto};

use crate::{ApiError, ZealotClient, api::seg, types::AttributeKindDto};

impl ZealotClient {
    pub async fn list_attribute_kinds(&self) -> Result<Vec<AttributeKindDto>, ApiError> {
        self.get("/attribute").await
    }

    pub async fn get_attribute_kind_by_key(&self, key: &str) -> Result<AttributeKindDto, ApiError> {
        self.get(&format!("/attribute/key/{}", seg(key))).await
    }

    pub async fn add_attribute_kind(
        &self,
        dto: &AddAttributeKindDto,
    ) -> Result<AttributeKindDto, ApiError> {
        self.post("/attribute", dto).await
    }

    pub async fn update_attribute_kind(
        &self,
        dto: &UpdateAttributeKindDto,
    ) -> Result<AttributeKindDto, ApiError> {
        self.patch(&format!("/attribute/id/{}", dto.kind_id), dto)
            .await
    }

    pub async fn delete_attribute_kind(&self, key: &str, force: bool) -> Result<(), ApiError> {
        self.delete(&format!("/attribute/key/{}?force={force}", seg(key)))
            .await
    }
}
