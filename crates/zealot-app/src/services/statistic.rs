use std::{collections::BTreeMap, sync::Arc};

use chrono::{DateTime, Utc};
use zealot_domain::{
    account::Account,
    attribute::{Attribute, AttributeScalar},
    common::id::Id,
    item::Item,
    statistic::{
        CreateStatisticEntryDto, DailyAggregation, StatisticDailyPointDto, StatisticEntry,
        StatisticEntryDto, StatisticEntryPageDto, StatisticSummaryDto, StatisticSummaryPointDto,
        UpdateStatisticEntryDto,
    },
};

use crate::repos::{common::RepoError, statistic::StatisticRepo};

use super::item::{ItemService, ItemServiceError};

#[derive(Debug)]
pub struct StatisticService {
    repo: Arc<dyn StatisticRepo>,
    item_service: Arc<ItemService>,
}

#[derive(Debug, thiserror::Error)]
pub enum StatisticServiceError {
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    Invalid(String),
    #[error("repo error: {0}")]
    Repo(#[from] RepoError),
}

impl StatisticService {
    pub fn new(repo: &Arc<dyn StatisticRepo>, item_service: &Arc<ItemService>) -> Self {
        Self {
            repo: repo.clone(),
            item_service: item_service.clone(),
        }
    }

    pub fn list_items(
        &self,
        parent_id: Option<Id>,
        account: &Account,
    ) -> Result<Vec<Item>, StatisticServiceError> {
        if let Some(parent_id) = parent_id {
            self.require_item(parent_id, account)?;
        }
        let mut items = self
            .item_service
            .get_items_by_type("Statistic", account)
            .map_err(map_item_error)?;
        if let Some(parent_id) = parent_id {
            items.retain(|item| item.parent_ids().contains(&parent_id));
        }
        Ok(items)
    }

    pub fn list_entries(
        &self,
        item_id: Id,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
        account: &Account,
    ) -> Result<StatisticEntryPageDto, StatisticServiceError> {
        self.validate_range(start, end)?;
        self.require_statistic(item_id, account)?;
        let mut entries = self
            .repo
            .list(item_id, start, end, limit + 1, offset, account)?;
        let has_more = entries.len() > limit as usize;
        entries.truncate(limit as usize);
        let count = entries.len();
        Ok(StatisticEntryPageDto {
            entries: entries.iter().map(StatisticEntryDto::from).collect(),
            count,
            next_offset: has_more.then_some(offset + count as i64),
        })
    }

    pub fn daily(
        &self,
        item_id: Id,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        account: &Account,
    ) -> Result<Vec<StatisticDailyPointDto>, StatisticServiceError> {
        self.validate_range(start, end)?;
        let item = self.require_statistic(item_id, account)?;
        let aggregation = daily_aggregation(&item)?;
        if aggregation == DailyAggregation::None {
            return Ok(Vec::new());
        }
        let entries = self.repo.list_all(item_id, start, end, account)?;
        let mut days: BTreeMap<_, Vec<&StatisticEntry>> = BTreeMap::new();
        for entry in &entries {
            days.entry(entry.occurred_at.date_naive())
                .or_default()
                .push(entry);
        }
        days.into_iter()
            .map(|(date, entries)| {
                let count = entries.len();
                let value = aggregate(&entries, aggregation)?;
                Ok(StatisticDailyPointDto {
                    date: date.to_string(),
                    value,
                    count,
                })
            })
            .collect()
    }

    pub fn summary(
        &self,
        item_id: Id,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        account: &Account,
    ) -> Result<StatisticSummaryDto, StatisticServiceError> {
        self.validate_range(start, end)?;
        self.require_statistic(item_id, account)?;
        let entries = self.repo.list_all(item_id, start, end, account)?;
        Ok(summarize(&entries)?)
    }

    pub fn create(
        &self,
        item_id: Id,
        dto: &CreateStatisticEntryDto,
        account: &Account,
    ) -> Result<StatisticEntry, StatisticServiceError> {
        self.require_statistic(item_id, account)?;
        validate_value(dto.value)?;
        let occurred_at = match &dto.occurred_at {
            Some(value) => parse_timestamp(value)?,
            None => Utc::now(),
        };
        let related_item_id = parse_related(dto.related_item_id)?;
        if let Some(related_item_id) = related_item_id {
            self.require_item(related_item_id, account)?;
        }
        let entry = self.repo.create(
            item_id,
            dto.value,
            occurred_at,
            related_item_id,
            normalize_comment(dto.comment.clone()),
            account,
        )?;
        tracing::info!(account_id = ?account.account_id, statistic_entry_id = ?entry.statistic_entry_id, item_id = ?item_id, "statistic entry created");
        Ok(entry)
    }

    pub fn update(
        &self,
        statistic_entry_id: Id,
        dto: &UpdateStatisticEntryDto,
        account: &Account,
    ) -> Result<StatisticEntry, StatisticServiceError> {
        let current = self
            .repo
            .get(statistic_entry_id, account)?
            .ok_or(StatisticServiceError::NotFound)?;
        self.require_statistic(current.item_id, account)?;
        let value = dto.value.unwrap_or(current.value);
        validate_value(value)?;
        let occurred_at = dto
            .occurred_at
            .as_deref()
            .map(parse_timestamp)
            .transpose()?
            .unwrap_or(current.occurred_at);
        let related_item_id =
            match dto.related_item_id {
                Some(Some(value)) => Some(Id::try_from(value).map_err(|_| {
                    StatisticServiceError::Invalid("invalid related_item_id".into())
                })?),
                Some(None) => None,
                None => current.related_item_id,
            };
        if let Some(related_item_id) = related_item_id {
            self.require_item(related_item_id, account)?;
        }
        let comment = match &dto.comment {
            Some(value) => normalize_comment(value.clone()),
            None => current.comment,
        };
        let entry = self
            .repo
            .update(
                statistic_entry_id,
                value,
                occurred_at,
                related_item_id,
                comment,
                account,
            )?
            .ok_or(StatisticServiceError::NotFound)?;
        tracing::info!(account_id = ?account.account_id, ?statistic_entry_id, "statistic entry updated");
        Ok(entry)
    }

    pub fn delete(
        &self,
        statistic_entry_id: Id,
        account: &Account,
    ) -> Result<(), StatisticServiceError> {
        let current = self
            .repo
            .get(statistic_entry_id, account)?
            .ok_or(StatisticServiceError::NotFound)?;
        self.require_statistic(current.item_id, account)?;
        if !self.repo.delete(statistic_entry_id, account)? {
            return Err(StatisticServiceError::NotFound);
        }
        tracing::info!(account_id = ?account.account_id, ?statistic_entry_id, "statistic entry deleted");
        Ok(())
    }

    fn require_item(&self, item_id: Id, account: &Account) -> Result<Item, StatisticServiceError> {
        self.item_service
            .get_item_by_id(&item_id, account)
            .map_err(map_item_error)?
            .ok_or(StatisticServiceError::NotFound)
    }

    fn require_statistic(
        &self,
        item_id: Id,
        account: &Account,
    ) -> Result<Item, StatisticServiceError> {
        let item = self.require_item(item_id, account)?;
        if !item.types.iter().any(|kind| kind.name == "Statistic") {
            return Err(StatisticServiceError::Invalid(
                "item is not assigned the Statistic type".into(),
            ));
        }
        for key in ["Value Kind", "Unit", "Daily Aggregation"] {
            if !item.attributes.contains_key(key) {
                return Err(StatisticServiceError::Invalid(format!(
                    "Statistic item is missing required attribute '{key}'"
                )));
            }
        }
        daily_aggregation(&item)?;
        Ok(item)
    }

    fn validate_range(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Result<(), StatisticServiceError> {
        if matches!((start, end), (Some(start), Some(end)) if end < start) {
            return Err(StatisticServiceError::Invalid(
                "end must not be before start".into(),
            ));
        }
        Ok(())
    }
}

fn map_item_error(error: ItemServiceError) -> StatisticServiceError {
    match error {
        ItemServiceError::Repo(error) => StatisticServiceError::Repo(error),
        ItemServiceError::NotFound | ItemServiceError::Unauthorized => {
            StatisticServiceError::NotFound
        }
        other => StatisticServiceError::Invalid(other.to_string()),
    }
}

fn parse_timestamp(value: &str) -> Result<DateTime<Utc>, StatisticServiceError> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|_| StatisticServiceError::Invalid(format!("invalid RFC 3339 timestamp: {value}")))
}

fn parse_related(value: Option<i64>) -> Result<Option<Id>, StatisticServiceError> {
    value
        .map(|value| {
            Id::try_from(value)
                .map_err(|_| StatisticServiceError::Invalid("invalid related_item_id".into()))
        })
        .transpose()
}

fn validate_value(value: f64) -> Result<(), StatisticServiceError> {
    if !value.is_finite() {
        return Err(StatisticServiceError::Invalid(
            "value must be finite".into(),
        ));
    }
    Ok(())
}

fn normalize_comment(comment: Option<String>) -> Option<String> {
    comment.and_then(|value| (!value.trim().is_empty()).then_some(value))
}

fn daily_aggregation(item: &Item) -> Result<DailyAggregation, StatisticServiceError> {
    match item.attributes.get("Daily Aggregation") {
        Some(Attribute::Scalar(AttributeScalar::Dropdown(value))) => {
            DailyAggregation::try_from(value.as_str()).map_err(StatisticServiceError::Invalid)
        }
        _ => Err(StatisticServiceError::Invalid(
            "Statistic item has an invalid Daily Aggregation attribute".into(),
        )),
    }
}

fn summarize(entries: &[StatisticEntry]) -> Result<StatisticSummaryDto, StatisticServiceError> {
    if entries.is_empty() {
        return Ok(StatisticSummaryDto {
            count: 0,
            first: None,
            latest: None,
            minimum: None,
            maximum: None,
            average: None,
            sum: None,
            delta: None,
        });
    }
    let first = entries.first().expect("not empty");
    let latest = entries.last().expect("not empty");
    let sum: f64 = entries.iter().map(|entry| entry.value).sum();
    if !sum.is_finite() {
        return Err(StatisticServiceError::Invalid(
            "aggregate is outside the finite numeric range".into(),
        ));
    }
    let delta = (entries.len() >= 2).then(|| latest.value - first.value);
    if delta.is_some_and(|value| !value.is_finite()) {
        return Err(StatisticServiceError::Invalid(
            "delta is outside the finite numeric range".into(),
        ));
    }
    Ok(StatisticSummaryDto {
        count: entries.len(),
        first: Some(StatisticSummaryPointDto {
            value: first.value,
            occurred_at: first.occurred_at.to_rfc3339(),
        }),
        latest: Some(StatisticSummaryPointDto {
            value: latest.value,
            occurred_at: latest.occurred_at.to_rfc3339(),
        }),
        minimum: Some(
            entries
                .iter()
                .map(|entry| entry.value)
                .fold(f64::INFINITY, f64::min),
        ),
        maximum: Some(
            entries
                .iter()
                .map(|entry| entry.value)
                .fold(f64::NEG_INFINITY, f64::max),
        ),
        average: Some(sum / entries.len() as f64),
        sum: Some(sum),
        delta,
    })
}

fn aggregate(
    entries: &[&StatisticEntry],
    aggregation: DailyAggregation,
) -> Result<f64, StatisticServiceError> {
    let sum: f64 = entries.iter().map(|entry| entry.value).sum();
    let value = match aggregation {
        DailyAggregation::Sum => sum,
        DailyAggregation::Average => sum / entries.len() as f64,
        DailyAggregation::Minimum => entries
            .iter()
            .map(|entry| entry.value)
            .fold(f64::INFINITY, f64::min),
        DailyAggregation::Maximum => entries
            .iter()
            .map(|entry| entry.value)
            .fold(f64::NEG_INFINITY, f64::max),
        DailyAggregation::Latest => entries.last().expect("group is not empty").value,
        DailyAggregation::None => {
            return Err(StatisticServiceError::Invalid(
                "Daily Aggregation None has no aggregate value".into(),
            ));
        }
    };
    if !value.is_finite() {
        return Err(StatisticServiceError::Invalid(
            "aggregate is outside the finite numeric range".into(),
        ));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::{aggregate, summarize};
    use chrono::{TimeZone, Utc};
    use zealot_domain::{common::id::Id, statistic::StatisticEntry};

    fn entry(id: i64, value: f64, hour: u32) -> StatisticEntry {
        let at = Utc.with_ymd_and_hms(2026, 8, 1, hour, 0, 0).unwrap();
        StatisticEntry {
            statistic_entry_id: Id::try_from(id).unwrap(),
            item_id: Id::try_from(1).unwrap(),
            value,
            occurred_at: at,
            related_item_id: None,
            comment: None,
            created_at: at,
            updated_at: at,
        }
    }

    #[test]
    fn empty_summary_is_explicit() {
        let summary = summarize(&[]).unwrap();
        assert_eq!(summary.count, 0);
        assert!(summary.sum.is_none());
        assert!(summary.delta.is_none());
    }

    #[test]
    fn summary_uses_chronological_first_and_latest() {
        let summary = summarize(&[entry(1, 3.0, 8), entry(2, 8.0, 9), entry(3, 5.0, 10)]).unwrap();
        assert_eq!(summary.count, 3);
        assert_eq!(summary.sum, Some(16.0));
        assert_eq!(summary.minimum, Some(3.0));
        assert_eq!(summary.maximum, Some(8.0));
        assert_eq!(summary.delta, Some(2.0));
    }

    #[test]
    fn every_daily_aggregation_mode_is_consistent() {
        use zealot_domain::statistic::DailyAggregation;
        let entries = [entry(1, 3.0, 8), entry(2, 8.0, 9), entry(3, 5.0, 10)];
        let refs = entries.iter().collect::<Vec<_>>();
        assert_eq!(aggregate(&refs, DailyAggregation::Sum).unwrap(), 16.0);
        assert_eq!(
            aggregate(&refs, DailyAggregation::Average).unwrap(),
            16.0 / 3.0
        );
        assert_eq!(aggregate(&refs, DailyAggregation::Minimum).unwrap(), 3.0);
        assert_eq!(aggregate(&refs, DailyAggregation::Maximum).unwrap(), 8.0);
        assert_eq!(aggregate(&refs, DailyAggregation::Latest).unwrap(), 5.0);
        assert!(aggregate(&refs, DailyAggregation::None).is_err());
    }
}
