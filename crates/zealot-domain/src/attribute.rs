use std::{collections::HashMap, fmt::Display};

use chrono::{NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::common::id::{Id, IdError};

#[derive(Debug, Clone)]
pub struct AttributeKind {
    pub kind_id: Id,
    pub is_system: bool,
    pub key: String,
    pub description: String,
    pub base_type: AttributeBaseType,
    pub spec: AttributeKindSpec,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeBaseScalarType {
    Text,
    Integer,
    Decimal,
    Date,
    Week,
    Dropdown,
    Boolean,
    Item,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeBaseType {
    Scalar(AttributeBaseScalarType),
    List(AttributeBaseScalarType),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeScalar {
    Text(String),
    Integer(i64),
    Decimal(f64),
    Date(NaiveDate),
    Week(Week),
    Dropdown(String),
    Boolean(bool),
    Item(Id),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Week {
    pub year: u64,
    pub week: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Attribute {
    Scalar(AttributeScalar),
    List(Vec<AttributeScalar>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddAttributeKindDto {
    key: String,
    description: String,
    base_type: String,
    config: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateAttributeKindDto {
    kind_id: i64,
    key: Option<String>,
    description: Option<String>,
    base_type: Option<String>,
    config: Option<Value>,
}

#[derive(Debug, Clone)]
pub enum AttributeKindSpec {
    Text {
        min_len: Option<usize>,
        max_len: Option<usize>,
        pattern: Option<String>,
    },
    Integer {
        min: Option<i64>,
        max: Option<i64>,
    },
    Decimal {
        min: Option<f64>,
        max: Option<f64>,
    },
    Date,
    Week,
    Dropdown {
        values: Vec<String>,
    },
    Boolean,
    List {
        list_type: AttributeBaseType,
    },
}

#[derive(Debug, Clone)]
pub enum AttributeFilterOp {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqualTo,
    LessThanOrEqualTo,
    LikeCaseInsensitive,
}

#[derive(Debug, Clone)]
pub enum AttributeListMode {
    Any, All, None
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AttributeFilterDto {
    pub key: String,
    pub op: String,
    pub value: Value,
    pub list_mode: String,
}

#[derive(Debug, Clone)]
pub struct AttributeFilter {
    pub key: String,
    pub op: AttributeFilterOp,
    pub value: Value,
    pub list_mode: AttributeListMode,
}

#[derive(Debug, thiserror::Error)]
pub enum AttributeError {
    #[error("value must be a boolean 'true' or 'false'")]
    InvalidBoolean,

    #[error("invalid value: {err_str:?}")]
    InvalidValue { err_str: String },

    #[error("value not in dropdown")]
    ValueNotInDropdown,

    #[error("kind not correctly configured")]
    KindError { err_str: String },

    #[error("value is not an integer")]
    NotAnInteger,

    #[error("number is below minimum")]
    BelowMin,

    #[error("number is above maximum")]
    AboveMax,

    #[error("not a floating point number")]
    NotAFloat,

    #[error("value must be a JSON array")]
    NotAList,

    #[error("not a week")]
    NotAWeek,

    #[error("not a valid item id: {err:?}")]
    NotAValidItemId { err: IdError },

    #[error("please create an attribute kind to support this: {err_str:?}")]
    RequestToMakeAttributeKind { err_str: String },

    #[error("type is not supported: {err_str:?}")]
    NotSupported { err_str: String },
}

impl Attribute {
    pub fn from_json(
        value: &Value,
        kinds: &HashMap<String, AttributeKind>,
    ) -> Result<HashMap<String, Attribute>, AttributeError> {
        // Takes all attribute kinds for current user
        // and parses each attribute
        let mut ret: HashMap<String, Attribute> = HashMap::new();

        // We must have an object with flat keys
        if let Value::Object(obj) = value {
            for (key, value) in obj {
                if kinds.contains_key(key) {
                    match Attribute::single_from_json(value, kinds.get(key).unwrap()) {
                        Ok(v) => {
                            ret.insert(key.clone(), v);
                        }
                        Err(err) => return Err(err),
                    }
                } else {
                    match Attribute::single_from_json_without_kind(value) {
                        Ok(v) => {
                            ret.insert(key.clone(), v);
                        }
                        Err(err) => return Err(err),
                    }
                }
            }
        } else {
            return Err(AttributeError::InvalidValue {
                err_str: String::from("must pass an object of attributes"),
            });
        }

        return Ok(ret);
    }

    pub fn single_from_json(value: &Value, kind: &AttributeKind) -> Result<Self, AttributeError> {
        match &kind.base_type {
            AttributeBaseType::Scalar(scalar_type) => {
                Self::parse_scalar(value, scalar_type, &kind.spec).map(Attribute::Scalar)
            }
            AttributeBaseType::List(scalar_type) => {
                let values = value.as_array().ok_or(AttributeError::NotAList)?;
                values
                    .iter()
                    .map(|value| Self::parse_scalar(value, scalar_type, &kind.spec))
                    .collect::<Result<Vec<_>, _>>()
                    .map(Attribute::List)
            }
        }
    }

    pub fn single_from_json_without_kind(value: &Value) -> Result<Self, AttributeError> {
        match value {
            Value::Array(_) => Err(AttributeError::RequestToMakeAttributeKind {
                err_str: String::from("array type requires attribute kind 'list'"),
            }),
            Value::Bool(b) => Ok(Attribute::Scalar(AttributeScalar::Boolean(*b))),
            Value::Null => Err(AttributeError::InvalidValue {
                err_str: String::from("cannot add null"),
            }),
            Value::Number(n) => {
                // First try int
                if n.is_i64() {
                    return Ok(Attribute::Scalar(AttributeScalar::Integer(
                        n.as_i64().unwrap(),
                    )));
                }

                if n.is_f64() {
                    return Ok(Attribute::Scalar(AttributeScalar::Decimal(
                        n.as_f64().unwrap(),
                    )));
                }

                Err(AttributeError::InvalidValue {
                    err_str: String::from("could not convert to number"),
                })
            }
            Value::Object(_) => Err(AttributeError::NotSupported {
                err_str: String::from("object types not supported"),
            }),
            Value::String(s) => Ok(Attribute::Scalar(AttributeScalar::Text(s.clone()))),
        }
    }

    fn parse_scalar(
        value: &Value,
        scalar_type: &AttributeBaseScalarType,
        spec: &AttributeKindSpec,
    ) -> Result<AttributeScalar, AttributeError> {
        match scalar_type {
            AttributeBaseScalarType::Text => Self::parse_text(value),
            AttributeBaseScalarType::Integer => Self::parse_integer(value, spec),
            AttributeBaseScalarType::Decimal => Self::parse_decimal(value, spec),
            AttributeBaseScalarType::Date => Self::parse_date(value),
            AttributeBaseScalarType::Week => Self::parse_week(value),
            AttributeBaseScalarType::Dropdown => Self::parse_dropdown(value, spec),
            AttributeBaseScalarType::Boolean => Self::parse_boolean(value),
            AttributeBaseScalarType::Item => Self::parse_item(value),
        }
    }

    fn parse_text(value: &Value) -> Result<AttributeScalar, AttributeError> {
        match value {
            Value::String(text) => Ok(AttributeScalar::Text(text.clone())),
            _ => Err(AttributeError::InvalidValue {
                err_str: String::from("must be a string"),
            }),
        }
    }

    fn parse_boolean(value: &Value) -> Result<AttributeScalar, AttributeError> {
        match value {
            Value::Bool(boolean) => Ok(AttributeScalar::Boolean(*boolean)),
            _ => Err(AttributeError::InvalidBoolean),
        }
    }

    fn parse_date(value: &Value) -> Result<AttributeScalar, AttributeError> {
        match value {
            Value::String(date) => chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .map(AttributeScalar::Date)
                .map_err(|err| AttributeError::InvalidValue {
                    err_str: format!("error parsing date: {err}"),
                }),
            _ => Err(AttributeError::InvalidValue {
                err_str: String::from("date must be a string with format YYYY-MM-DD"),
            }),
        }
    }

    fn parse_dropdown(
        value: &Value,
        spec: &AttributeKindSpec,
    ) -> Result<AttributeScalar, AttributeError> {
        let selected = match value {
            Value::String(selected) => selected,
            _ => {
                return Err(AttributeError::InvalidValue {
                    err_str: String::from("must be a string of an accepted value"),
                });
            }
        };

        match spec {
            AttributeKindSpec::Dropdown { values } => {
                if values.contains(selected) {
                    Ok(AttributeScalar::Dropdown(selected.clone()))
                } else {
                    Err(AttributeError::ValueNotInDropdown)
                }
            }
            _ => Err(AttributeError::KindError {
                err_str: String::from("kind does not contain dropdown spec"),
            }),
        }
    }

    fn parse_integer(
        value: &Value,
        spec: &AttributeKindSpec,
    ) -> Result<AttributeScalar, AttributeError> {
        let int = match value {
            Value::Number(number) => number.as_i64().ok_or(AttributeError::NotAnInteger)?,
            _ => return Err(AttributeError::NotAnInteger),
        };

        if let AttributeKindSpec::Integer { min, max } = spec {
            if min.is_some_and(|min| min > int) {
                return Err(AttributeError::BelowMin);
            }

            if max.is_some_and(|max| max < int) {
                return Err(AttributeError::AboveMax);
            }
        }

        Ok(AttributeScalar::Integer(int))
    }

    fn parse_decimal(
        value: &Value,
        spec: &AttributeKindSpec,
    ) -> Result<AttributeScalar, AttributeError> {
        let float = match value {
            Value::Number(number) => number.as_f64().ok_or(AttributeError::NotAFloat)?,
            _ => return Err(AttributeError::NotAFloat),
        };

        if let AttributeKindSpec::Decimal { min, max } = spec {
            if min.is_some_and(|min| min > float) {
                return Err(AttributeError::BelowMin);
            }

            if max.is_some_and(|max| max < float) {
                return Err(AttributeError::AboveMax);
            }
        }

        Ok(AttributeScalar::Decimal(float))
    }

    fn parse_week(value: &Value) -> Result<AttributeScalar, AttributeError> {
        match value {
            Value::String(week) => Week::try_from(week.as_str()).map(AttributeScalar::Week),
            _ => Err(AttributeError::NotAWeek),
        }
    }

    fn parse_item(value: &Value) -> Result<AttributeScalar, AttributeError> {
        let id = match value {
            Value::Number(number) => number.as_i64().ok_or(AttributeError::InvalidValue {
                err_str: String::from("must be an id"),
            })?,
            _ => {
                return Err(AttributeError::InvalidValue {
                    err_str: String::from("must be an integer"),
                });
            }
        };

        let id: Id = id
            .try_into()
            .map_err(|err| AttributeError::NotAValidItemId { err })?;

        Ok(AttributeScalar::Item(id))
    }
}

impl TryFrom<&str> for Week {
    type Error = AttributeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        let (year_str, week_str) = trimmed.split_once("-W").ok_or(AttributeError::NotAWeek)?;

        if year_str.len() != 4 || week_str.len() != 2 {
            return Err(AttributeError::NotAWeek);
        }

        let year_i32 = year_str
            .parse::<i32>()
            .map_err(|_| AttributeError::NotAWeek)?;
        let year = u64::try_from(year_i32).map_err(|_| AttributeError::NotAWeek)?;

        let week = week_str
            .parse::<u8>()
            .map_err(|_| AttributeError::NotAWeek)?;

        if week == 0 {
            return Err(AttributeError::NotAWeek);
        }

        NaiveDate::from_isoywd_opt(year_i32, week.into(), Weekday::Mon)
            .ok_or(AttributeError::NotAWeek)?;

        Ok(Self { year, week })
    }
}

impl Display for Week {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-W{}", self.year, self.week)
    }
}

impl TryFrom<&AttributeFilterDto> for AttributeFilter {
    type Error = String;

    fn try_from(value: &AttributeFilterDto) -> Result<Self, Self::Error> {
        Ok(Self {
            key: value.key.clone(),
            op: AttributeFilterOp::try_from(value.op.as_str())
                .map_err(|e| e)?,
            value: value.value.clone(),
            list_mode: AttributeListMode::try_from(value.list_mode.as_str())
                .map_err(|e| e)?,
        })
    }
}

impl TryFrom<&str> for AttributeFilterOp {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "eq" => Ok(Self::Equal),
            "=" => Ok(Self::Equal),
            "ne" => Ok(Self::NotEqual),
            "!=" => Ok(Self::NotEqual),
            "<>" => Ok(Self::NotEqual),
            "gt" => Ok(Self::GreaterThan),
            ">" => Ok(Self::GreaterThan),
            "lt" => Ok(Self::LessThan),
            "<" => Ok(Self::LessThan),
            "gte" => Ok(Self::GreaterThanOrEqualTo),
            ">=" => Ok(Self::GreaterThanOrEqualTo),
            "lte" => Ok(Self::LessThanOrEqualTo),
            "<=" => Ok(Self::LessThanOrEqualTo),
            _ => Err(format!("{} operation not supported", value))
        }
    }
}

impl TryFrom<&str> for AttributeListMode {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "any" => Ok(Self::Any),
            "all" => Ok(Self::All),
            "none" => Ok(Self::None),
            _ => Err(format!("{} list mode not supported", value))
        }
    }
}

#[cfg(test)]
mod attribute_tests {
    use serde_json::json;

    use super::*;

    fn kind(base_type: AttributeBaseType, spec: AttributeKindSpec) -> AttributeKind {
        AttributeKind {
            kind_id: Id::try_from(1).unwrap(),
            is_system: false,
            key: String::from("test"),
            description: String::from("test"),
            base_type,
            spec,
        }
    }

    #[test]
    fn parses_list_values_with_scalar_rules() {
        let kind = kind(
            AttributeBaseType::List(AttributeBaseScalarType::Integer),
            AttributeKindSpec::Integer {
                min: Some(1),
                max: Some(5),
            },
        );

        let value = json!([1, 2, 5]);

        let attribute = Attribute::single_from_json(&value, &kind).unwrap();

        assert_eq!(
            attribute,
            Attribute::List(vec![
                AttributeScalar::Integer(1),
                AttributeScalar::Integer(2),
                AttributeScalar::Integer(5),
            ])
        );
    }

    #[test]
    fn rejects_non_arrays_for_list_values() {
        let kind = kind(
            AttributeBaseType::List(AttributeBaseScalarType::Boolean),
            AttributeKindSpec::Boolean,
        );

        let err = Attribute::single_from_json(&json!(true), &kind).unwrap_err();

        assert!(matches!(err, AttributeError::NotAList));
    }

    #[test]
    fn rejects_invalid_list_elements_using_scalar_validation() {
        let kind = kind(
            AttributeBaseType::List(AttributeBaseScalarType::Integer),
            AttributeKindSpec::Integer {
                min: Some(1),
                max: Some(5),
            },
        );

        let err = Attribute::single_from_json(&json!([1, 6]), &kind).unwrap_err();

        assert!(matches!(err, AttributeError::AboveMax));
    }

    #[test]
    fn parses_text_without_json_quoting() {
        let kind = kind(
            AttributeBaseType::Scalar(AttributeBaseScalarType::Text),
            AttributeKindSpec::Text {
                min_len: None,
                max_len: None,
                pattern: None,
            },
        );

        let attribute = Attribute::single_from_json(&json!("hello"), &kind).unwrap();

        assert_eq!(
            attribute,
            Attribute::Scalar(AttributeScalar::Text(String::from("hello")))
        );
    }
}

impl Serialize for Attribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Attribute::Scalar(value) => match value {
                AttributeScalar::Text(text) => serializer.serialize_str(text),
                AttributeScalar::Integer(integer) => serializer.serialize_i64(*integer),
                AttributeScalar::Decimal(decimal) => serializer.serialize_f64(*decimal),
                AttributeScalar::Date(date) => {
                    let formatted = date.format("%Y-%m-%d").to_string();
                    serializer.serialize_str(&formatted)
                }
                AttributeScalar::Week(week) => {
                    let formatted = format!("{:04}-W{:02}", week.year, week.week);
                    serializer.serialize_str(&formatted)
                }
                AttributeScalar::Dropdown(selected) => serializer.serialize_str(selected),
                AttributeScalar::Boolean(boolean) => serializer.serialize_bool(*boolean),
                AttributeScalar::Item(id) => serializer.serialize_i64((*id).into()),
            },
            Attribute::List(values) => {
                use serde::ser::SerializeSeq;

                let mut seq = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    match value {
                        AttributeScalar::Text(text) => seq.serialize_element(text)?,
                        AttributeScalar::Integer(integer) => seq.serialize_element(integer)?,
                        AttributeScalar::Decimal(decimal) => seq.serialize_element(decimal)?,
                        AttributeScalar::Date(date) => {
                            let formatted = date.format("%Y-%m-%d").to_string();
                            seq.serialize_element(&formatted)?;
                        }
                        AttributeScalar::Week(week) => {
                            let formatted = format!("{:04}-W{:02}", week.year, week.week);
                            seq.serialize_element(&formatted)?;
                        }
                        AttributeScalar::Dropdown(selected) => seq.serialize_element(selected)?,
                        AttributeScalar::Boolean(boolean) => seq.serialize_element(boolean)?,
                        AttributeScalar::Item(id) => {
                            let item_id: i64 = (*id).into();
                            seq.serialize_element(&item_id)?;
                        }
                    }
                }
                seq.end()
            }
        }
    }
}
