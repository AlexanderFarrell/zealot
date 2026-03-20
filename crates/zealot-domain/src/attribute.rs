use std::any;

use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, IsoWeek};
use serde_json::Value;

use crate::{common::id::Id, item::Item};

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
    Week(IsoWeek),
    Dropdown(String),
    Boolean(bool),
    Item(Item),
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
    NotAnInteger
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
                        match kind.spec {
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
                        match n.as_i64()



                        if !n.is_i64() {
                            Err(AttributeError::NotAnInteger)
                        }

                        let int = n.as_i64()
                    }
                }
            }
            
        }
    }
}