use std::collections::HashMap;

use serde_json::{Value, json};
use zealot_domain::{
    attribute::AttributeFilterDto,
    item::{
        AddItemDto, ItemDto, ItemQueryDto, ItemQueryPageDto, SearchResultDto, SearchScope,
        UpdateItemDto,
    },
    item_content::{ContentEditDto, ContentWriteDto, GrepPageDto, GrepQueryDto},
};

use crate::{
    ApiError, ZealotClient,
    api::seg,
    http::Download,
    types::{MostViewedItemDto, RebuildLinksResultDto},
};

impl ZealotClient {
    pub async fn get_item(&self, item_id: i64) -> Result<ItemDto, ApiError> {
        self.get(&format!("/item/id/{item_id}")).await
    }

    pub async fn get_item_by_title(&self, title: &str) -> Result<ItemDto, ApiError> {
        self.get(&format!("/item/title/{}", seg(title))).await
    }

    /// Root items, or all items of a type when `item_type` is given.
    pub async fn list_items(&self, item_type: Option<&str>) -> Result<Vec<ItemDto>, ApiError> {
        let path = match item_type {
            Some(t) => format!("/item?type={}", seg(t)),
            None => "/item".to_string(),
        };
        self.get(&path).await
    }

    pub async fn recent_items(&self, limit: i64, offset: i64) -> Result<Vec<ItemDto>, ApiError> {
        self.get(&format!("/item/recent?limit={limit}&offset={offset}"))
            .await
    }

    pub async fn random_items(&self, count: usize) -> Result<Vec<ItemDto>, ApiError> {
        self.get(&format!("/item/random?count={count}")).await
    }

    pub async fn search_items(
        &self,
        term: &str,
        scope: SearchScope,
        regex: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SearchResultDto>, ApiError> {
        let scope = match scope {
            SearchScope::Title => "title",
            SearchScope::Content => "content",
            SearchScope::Heading => "heading",
        };
        self.get(&format!(
            "/item/search?term={}&scope={scope}&regex={regex}&limit={limit}&offset={offset}",
            seg(term)
        ))
        .await
    }

    pub async fn get_children(&self, item_id: i64) -> Result<Vec<ItemDto>, ApiError> {
        self.get(&format!("/item/children/{item_id}")).await
    }

    pub async fn get_related(&self, item_id: i64) -> Result<Vec<ItemDto>, ApiError> {
        self.get(&format!("/item/related/{item_id}")).await
    }

    pub async fn get_backlinks(&self, item_id: i64) -> Result<Vec<ItemDto>, ApiError> {
        self.get(&format!("/item/backlinks/{item_id}")).await
    }

    pub async fn filter_items(
        &self,
        filters: &[AttributeFilterDto],
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ItemDto>, ApiError> {
        self.post(
            "/item/filter",
            &json!({ "filters": filters, "limit": limit, "offset": offset }),
        )
        .await
    }

    /// Query items by type, subtree, filters, and sort; returns one page and the total count.
    pub async fn query_items(&self, query: &ItemQueryDto) -> Result<ItemQueryPageDto, ApiError> {
        self.post("/item/query", query).await
    }

    /// Line-level search across item content, matched on the server.
    pub async fn grep_items(&self, query: &GrepQueryDto) -> Result<GrepPageDto, ApiError> {
        self.post("/item/grep", query).await
    }

    pub async fn add_item(&self, dto: &AddItemDto) -> Result<ItemDto, ApiError> {
        self.post("/item", dto).await
    }

    pub async fn update_item(&self, dto: &UpdateItemDto) -> Result<ItemDto, ApiError> {
        self.patch(&format!("/item/{}", dto.item_id), dto).await
    }

    /// Apply one targeted content edit on the server (exact replace, append,
    /// or section rewrite). A stale `expected_version` fails with HTTP 409.
    pub async fn edit_item_content(
        &self,
        item_id: i64,
        dto: &ContentEditDto,
    ) -> Result<ContentWriteDto, ApiError> {
        self.post(&format!("/item/{item_id}/content"), dto).await
    }

    pub async fn delete_item(&self, item_id: i64) -> Result<(), ApiError> {
        self.delete(&format!("/item/{item_id}")).await
    }

    /// Set (merge) attributes on an item. Values are validated server-side
    /// against the account's attribute kinds.
    pub async fn set_item_attributes(
        &self,
        item_id: i64,
        attrs: &HashMap<String, Value>,
    ) -> Result<(), ApiError> {
        self.patch_no_response(&format!("/item/{item_id}/attr"), attrs)
            .await
    }

    pub async fn rename_item_attribute(
        &self,
        item_id: i64,
        old_key: &str,
        new_key: &str,
    ) -> Result<(), ApiError> {
        self.patch_no_response(
            &format!("/item/{item_id}/attr/rename"),
            &json!({ "old_key": old_key, "new_key": new_key }),
        )
        .await
    }

    pub async fn delete_item_attribute(&self, item_id: i64, key: &str) -> Result<(), ApiError> {
        self.delete(&format!("/item/{item_id}/attr/{}", seg(key)))
            .await
    }

    pub async fn assign_type(&self, item_id: i64, type_name: &str) -> Result<(), ApiError> {
        self.post_empty(&format!("/item/{item_id}/assign_type/{}", seg(type_name)))
            .await
    }

    pub async fn unassign_type(&self, item_id: i64, type_name: &str) -> Result<(), ApiError> {
        self.delete(&format!("/item/{item_id}/assign_type/{}", seg(type_name)))
            .await
    }

    pub async fn rebuild_links(&self) -> Result<RebuildLinksResultDto, ApiError> {
        self.post("/item/rebuild-links", &json!({})).await
    }

    pub async fn export_item_pdf(&self, item_id: i64) -> Result<Download, ApiError> {
        self.get_bytes(&format!("/item/id/{item_id}/export/pdf"))
            .await
    }

    pub async fn export_item_docx(&self, item_id: i64) -> Result<Download, ApiError> {
        self.get_bytes(&format!("/item/id/{item_id}/export/docx"))
            .await
    }

    pub async fn most_viewed(&self, limit: i64) -> Result<Vec<MostViewedItemDto>, ApiError> {
        self.get(&format!("/analysis/most-viewed?limit={limit}"))
            .await
    }
}
