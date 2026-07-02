use zealot_domain::rule::{AddRuleDto, RuleDto, UpdateRuleDto};

use crate::{ApiError, ZealotClient, types::RuleRunResultDto};

impl ZealotClient {
    pub async fn list_rules(&self) -> Result<Vec<RuleDto>, ApiError> {
        self.get("/rule/").await
    }

    pub async fn get_rule(&self, rule_id: i64) -> Result<RuleDto, ApiError> {
        self.get(&format!("/rule/{rule_id}")).await
    }

    pub async fn add_rule(&self, dto: &AddRuleDto) -> Result<RuleDto, ApiError> {
        self.post("/rule/", dto).await
    }

    pub async fn update_rule(&self, rule_id: i64, dto: &UpdateRuleDto) -> Result<RuleDto, ApiError> {
        self.patch(&format!("/rule/{rule_id}"), dto).await
    }

    pub async fn delete_rule(&self, rule_id: i64) -> Result<(), ApiError> {
        self.delete(&format!("/rule/{rule_id}")).await
    }

    pub async fn run_rule(&self, rule_id: i64) -> Result<RuleRunResultDto, ApiError> {
        self.post(&format!("/rule/{rule_id}/run"), &serde_json::json!({}))
            .await
    }
}
