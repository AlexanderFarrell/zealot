use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

use crate::common::id::Id;

#[derive(Debug, Clone)]
pub struct StatisticEntry {
    pub statistic_entry_id: Id,
    pub item_id: Id,
    pub value: f64,
    pub occurred_at: DateTime<Utc>,
    pub related_item_id: Option<Id>,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub type StatisticEntryCore = StatisticEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticEntryDto {
    pub statistic_entry_id: i64,
    pub item_id: i64,
    pub value: f64,
    pub occurred_at: String,
    pub related_item_id: Option<i64>,
    pub comment: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStatisticEntryDto {
    pub value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occurred_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_item_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// A missing field leaves the value unchanged; an explicit JSON null clears it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateStatisticEntryDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occurred_at: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub related_item_id: Option<Option<i64>>,
    #[serde(
        default,
        deserialize_with = "deserialize_nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub comment: Option<Option<String>>,
}

fn deserialize_nullable<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticEntryPageDto {
    pub count: usize,
    pub next_offset: Option<i64>,
    pub entries: Vec<StatisticEntryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticDailyPointDto {
    pub date: String,
    pub value: f64,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticSummaryPointDto {
    pub value: f64,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticSummaryDto {
    pub count: usize,
    pub first: Option<StatisticSummaryPointDto>,
    pub latest: Option<StatisticSummaryPointDto>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub average: Option<f64>,
    pub sum: Option<f64>,
    pub delta: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DailyAggregation {
    Sum,
    Average,
    Minimum,
    Maximum,
    Latest,
    None,
}

impl TryFrom<&str> for DailyAggregation {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Sum" => Ok(Self::Sum),
            "Average" => Ok(Self::Average),
            "Minimum" => Ok(Self::Minimum),
            "Maximum" => Ok(Self::Maximum),
            "Latest" => Ok(Self::Latest),
            "None" => Ok(Self::None),
            other => Err(format!("invalid Daily Aggregation: {other}")),
        }
    }
}

impl From<&StatisticEntry> for StatisticEntryDto {
    fn from(value: &StatisticEntry) -> Self {
        Self {
            statistic_entry_id: value.statistic_entry_id.into(),
            item_id: value.item_id.into(),
            value: value.value,
            occurred_at: value.occurred_at.to_rfc3339(),
            related_item_id: value.related_item_id.map(i64::from),
            comment: value.comment.clone(),
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UpdateStatisticEntryDto;

    #[test]
    fn nullable_patch_fields_distinguish_missing_null_and_value() {
        let missing: UpdateStatisticEntryDto = serde_json::from_str("{}").unwrap();
        assert_eq!(missing.related_item_id, None);
        assert_eq!(missing.comment, None);

        let cleared: UpdateStatisticEntryDto =
            serde_json::from_str(r#"{"related_item_id":null,"comment":null}"#).unwrap();
        assert_eq!(cleared.related_item_id, Some(None));
        assert_eq!(cleared.comment, Some(None));

        let set: UpdateStatisticEntryDto =
            serde_json::from_str(r#"{"related_item_id":42,"comment":"note"}"#).unwrap();
        assert_eq!(set.related_item_id, Some(Some(42)));
        assert_eq!(set.comment, Some(Some("note".into())));
    }
}
