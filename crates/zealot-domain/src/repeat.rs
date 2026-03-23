use std::fmt::Display;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::item::{Item, ItemDto};

pub struct RepeatEntry {
    pub status: RepeatStatus,
    pub item: Item,
    pub date: NaiveDate,
    pub comment: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatStatus {
    Complete,
    Skip,
    Alternate,
    NotComplete,
}

// Send DTOs

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RepeatEntryDto {
    status: String,
    item: ItemDto,
    date: String,
    comment: String,
}

// Update DTOs

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateRepeatEntryDto {
    pub item_id: i64,
    pub date: String,
    pub status: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum RepeatError {}

impl TryFrom<&str> for RepeatStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "Complete" => Ok(Self::Complete),
            "Skip" => Ok(Self::Skip),
            "Alternate" => Ok(Self::Alternate),
            "Not Complete" | "NotComplete" => Ok(Self::NotComplete),
            other => Err(format!("invalid repeat status: {other}")),
        }
    }
}

impl TryFrom<String> for RepeatStatus {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl Display for RepeatStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Complete => "Complete",
            Self::Skip => "Skip",
            Self::Alternate => "Alternate",
            Self::NotComplete => "Not Complete",
        };

        write!(f, "{value}")
    }
}

#[cfg(test)]
mod repeat_tests {
    use super::RepeatStatus;

    #[test]
    fn parses_repeat_status_wire_values() {
        assert_eq!(
            RepeatStatus::try_from("Complete").unwrap(),
            RepeatStatus::Complete
        );
        assert_eq!(RepeatStatus::try_from("Skip").unwrap(), RepeatStatus::Skip);
        assert_eq!(
            RepeatStatus::try_from("Alternate").unwrap(),
            RepeatStatus::Alternate
        );
        assert_eq!(
            RepeatStatus::try_from("Not Complete").unwrap(),
            RepeatStatus::NotComplete
        );
    }

    #[test]
    fn formats_repeat_status_wire_values() {
        assert_eq!(RepeatStatus::Complete.to_string(), "Complete");
        assert_eq!(RepeatStatus::Skip.to_string(), "Skip");
        assert_eq!(RepeatStatus::Alternate.to_string(), "Alternate");
        assert_eq!(RepeatStatus::NotComplete.to_string(), "Not Complete");
    }
}
