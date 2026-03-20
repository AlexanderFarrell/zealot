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

pub enum Attribute {
    Scalar(AttributeScalar),
    List(Vec<AttributeScalar>),
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
        list_type: AttributeBaseType
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AttributeError {

    #[error("value must be a boolean 'true' or 'false'")]
    InvalidBoolean,

    #[error("invalid value: {err_str:?}")]
    InvalidValue{err_str: String},

    #[error("value not in dropdown")]
    ValueNotInDropdown,

    #[error("kind not correctly configured")]
    KindError{err_str: String},

    #[error("value is not an integer")]
    NotAnInteger,

    #[error("number is below minimum")]
    BelowMin,

    #[error("number is above maximum")]
    AboveMax,

    #[error("not a floating point number")]
    NotAFloat,

    #[error("not a week")]
    NotAWeek,

    #[error("not a valid item id: {err:?}")]
    NotAValidItemId{err: IdError},
}

impl Attribute {
    pub fn from_json(value: &Value, kind: &AttributeKind) -> Result<Self, AttributeError> {
        match kind.base_type {
            AttributeBaseType::Scalar(AttributeBaseScalarType::Text) => {
                return Ok(Attribute::Scalar(AttributeScalar::Text(format!("{}", value))))
            },

            AttributeBaseType::Scalar(AttributeBaseScalarType::Boolean) => {
                match value {
                    Value::Bool(v) => return Ok(Attribute::Scalar(AttributeScalar::Boolean(*v))),
                    _ => Err(AttributeError::InvalidBoolean)
                }
            },

            AttributeBaseType::Scalar(AttributeBaseScalarType::Date) => {
                match value {
                    Value::String(v) => {
                        match chrono::NaiveDate::parse_from_str(v.as_str(), "%Y-%m-%d") {
                            Ok(d) => Ok(Attribute::Scalar(AttributeScalar::Date(d))),
                            Err(e) => Err(AttributeError::InvalidValue { err_str: format!("Error parsing date: {}", e.to_string()) })
                        }
                    },
                    _ => Err(AttributeError::InvalidValue{err_str: String::from("Date must be a string with format YYYY-MM-DD")})
                }
            },

            AttributeBaseType::Scalar(AttributeBaseScalarType::Dropdown) => {
                match value {
                    Value::String(v) => {
                        // Must be within the values
                        match &kind.spec {
                            AttributeKindSpec::Dropdown { values } => {
                                if values.contains(v) {
                                    Ok(Attribute::Scalar(
                                        AttributeScalar::Dropdown(v.clone())
                                    ))
                                } else {
                                    Err(AttributeError::ValueNotInDropdown)
                                }
                            },
                            _ => {
                                Err(AttributeError::KindError { err_str: String::from(
                                    "Kind does not contain dropdown spec") })
                            }
                        }
                    },
                    _ => Err(AttributeError::InvalidValue { err_str: String::from("Must be a string of an accepted value") })
                }
            },

            AttributeBaseType::Scalar(AttributeBaseScalarType::Integer) => {
                match value {
                    Value::Number(n) => {
                        let int = n.as_i64()
                            .ok_or_else(|| AttributeError::NotAnInteger)?;

                        // Not required but check
                        if let AttributeKindSpec::Integer { min, max } = &kind.spec {
                            if min.is_some() && min.unwrap() > int {
                                return Err(AttributeError::BelowMin)
                            }

                            if max.is_some() && max.unwrap() < int {
                                return Err(AttributeError::AboveMax)
                            }
                        }

                        return Ok(Attribute::Scalar(AttributeScalar::Integer(int)))
                    },
                    _ => Err(AttributeError::NotAnInteger)
                }
            },

            AttributeBaseType::Scalar(AttributeBaseScalarType::Decimal) => {
                match value {
                    Value::Number(n) => {
                        let float = n.as_f64()
                            .ok_or_else(|| AttributeError::NotAFloat)?;

                        if let AttributeKindSpec::Decimal { min, max } = &kind.spec {
                            if min.is_some() && min.unwrap() > float {
                                return Err(AttributeError::BelowMin);
                            }

                            if max.is_some() && max.unwrap() < float {
                                return Err(AttributeError::AboveMax);
                            }
                        }
                        return Ok(Attribute::Scalar(AttributeScalar::Decimal(float)));
                    },
                    _ => Err(AttributeError::NotAFloat)
                }
            },

            AttributeBaseType::Scalar(AttributeBaseScalarType::Week) => {
                match value {
                    Value::String(n) => {
                        let week = Week::try_from(n.as_str())?;
                        Ok(Attribute::Scalar(AttributeScalar::Week(week)))
                    },
                    _ => Err(AttributeError::NotAWeek)
                }
            },

            AttributeBaseType::Scalar(AttributeBaseScalarType::Item) => {
                match value {
                    // Match on ID
                    Value::Number(n) => {
                        let int = n.as_i64()
                            .ok_or_else(|| AttributeError::InvalidValue
                                 { err_str: String::from("must be an id") })?;

                        let id: Id = int.try_into()
                            .map_err(|e| AttributeError::NotAValidItemId { err: e })?;

                        Ok(Attribute::Scalar(AttributeScalar::Item(id)))
                    },
                    _ => Err(AttributeError::InvalidValue { err_str: String::from("must be an integer") })
                }
            },

            AttributeBaseType::List(_) => {
                Err(AttributeError::KindError {
                    err_str: String::from("List attributes are not yet supported"),
                })
            },
            
        }
    }
}

impl TryFrom<&str> for Week {
    type Error = AttributeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        let (year_str, week_str) = trimmed
            .split_once("-W")
            .ok_or(AttributeError::NotAWeek)?;

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
