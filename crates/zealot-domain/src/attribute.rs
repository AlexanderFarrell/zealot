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
    /// Email address, validated and stored as text.
    Email,
    /// Telephone number (digits with optional `+`, spaces, `-`, `.`, parentheses), stored as text.
    Telephone,
    /// Absolute `http`/`https` URL, stored as text.
    Url,
    /// Link to a media file: an absolute `http`/`https` URL or a path into the media store, stored as text.
    Media,
    /// Address text, or coordinates written `lat,lon`, stored as text.
    Place,
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
    Email(String),
    Telephone(String),
    Url(String),
    Media(String),
    Place(String),
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
    pub key: String,
    pub description: String,
    pub base_type: String,
    pub config: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateAttributeKindDto {
    pub kind_id: i64,
    pub key: Option<String>,
    pub description: Option<String>,
    pub base_type: Option<String>,
    pub config: Option<Value>,
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
    Item,
    Email,
    Telephone,
    Url,
    Media,
    Place,
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
    Any,
    All,
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AttributeFilterDto {
    pub key: String,
    pub op: String,
    pub value: Value,
    #[serde(default = "default_filter_list_mode")]
    pub list_mode: String,
}

fn default_filter_list_mode() -> String {
    String::from("any")
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

    /// Another error, tagged with the attribute it came from and the shape that attribute expects.
    #[error("attribute '{key}' (expects {expected}): {source}")]
    ForAttribute {
        key: String,
        expected: String,
        source: Box<AttributeError>,
    },
}

impl AttributeBaseScalarType {
    fn describe(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Integer => "integer",
            Self::Decimal => "decimal",
            Self::Date => "date YYYY-MM-DD",
            Self::Week => "week YYYY-Wnn",
            Self::Dropdown => "one of the dropdown values",
            Self::Boolean => "boolean",
            Self::Item => "item id",
            Self::Email => "email address",
            Self::Telephone => "telephone number",
            Self::Url => "http(s) URL",
            Self::Media => "media link (http(s) URL or /media path)",
            Self::Place => "address or 'lat,lon' coordinates",
        }
    }
}

impl AttributeError {
    /// Name the attribute (and the shape its kind expects, when known) in this error.
    pub fn for_attribute(self, key: &str, kind: Option<&AttributeKind>) -> Self {
        let expected = match kind.map(|kind| &kind.base_type) {
            Some(AttributeBaseType::Scalar(scalar)) => scalar.describe().to_string(),
            Some(AttributeBaseType::List(scalar)) => {
                format!("a JSON array of {}", scalar.describe())
            }
            None => String::from("an unregistered attribute kind"),
        };
        Self::ForAttribute {
            key: key.to_string(),
            expected,
            source: Box::new(self),
        }
    }
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
                    let kind = kinds.get(key).unwrap();
                    match Attribute::single_from_json(value, kind) {
                        Ok(v) => {
                            ret.insert(key.clone(), v);
                        }
                        Err(err) => return Err(err.for_attribute(key, Some(kind))),
                    }
                } else {
                    match Attribute::single_from_json_without_kind(value) {
                        Ok(v) => {
                            ret.insert(key.clone(), v);
                        }
                        Err(err) => return Err(err.for_attribute(key, None)),
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
            AttributeBaseScalarType::Email => {
                Self::parse_formatted(value, validate_email, "email address", AttributeScalar::Email)
            }
            AttributeBaseScalarType::Telephone => Self::parse_formatted(
                value,
                validate_telephone,
                "telephone number",
                AttributeScalar::Telephone,
            ),
            AttributeBaseScalarType::Url => {
                Self::parse_formatted(value, validate_url, "http(s) URL", AttributeScalar::Url)
            }
            AttributeBaseScalarType::Media => Self::parse_formatted(
                value,
                validate_media,
                "media link (http(s) URL or /media path)",
                AttributeScalar::Media,
            ),
            AttributeBaseScalarType::Place => Self::parse_formatted(
                value,
                validate_place,
                "address or 'lat,lon' coordinates",
                AttributeScalar::Place,
            ),
        }
    }

    /// Parse a string value that must satisfy `validate`, wrapping the trimmed text with `wrap`.
    fn parse_formatted(
        value: &Value,
        validate: fn(&str) -> bool,
        expected: &str,
        wrap: fn(String) -> AttributeScalar,
    ) -> Result<AttributeScalar, AttributeError> {
        let text = value.as_str().ok_or_else(|| AttributeError::InvalidValue {
            err_str: format!("must be a string containing a {expected}"),
        })?;
        let text = text.trim();
        if validate(text) {
            Ok(wrap(text.to_string()))
        } else {
            Err(AttributeError::InvalidValue {
                err_str: format!("'{text}' is not a valid {expected}"),
            })
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

/// True for `local@domain.tld` with no whitespace, a single `@`, and a dotted domain.
fn validate_email(text: &str) -> bool {
    let Some((local, domain)) = text.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !text.chars().any(char::is_whitespace)
        && !domain.contains('@')
        && domain.contains('.')
        && domain
            .split('.')
            .all(|label| !label.is_empty() && !label.starts_with('-') && !label.ends_with('-'))
}

/// True for 7-15 digits, optionally with a leading `+` and spaces, `-`, `.`, `(`, `)` separators.
fn validate_telephone(text: &str) -> bool {
    let body = text.strip_prefix('+').unwrap_or(text);
    let digits = body.chars().filter(char::is_ascii_digit).count();
    (7..=15).contains(&digits)
        && body
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, ' ' | '-' | '.' | '(' | ')'))
}

/// True for an absolute `http://` or `https://` URL with a host and no whitespace.
fn validate_url(text: &str) -> bool {
    let rest = text
        .strip_prefix("https://")
        .or_else(|| text.strip_prefix("http://"));
    rest.is_some_and(|rest| {
        !rest.is_empty()
            && !rest.starts_with('/')
            && !text.chars().any(char::is_whitespace)
    })
}

/// True for an http(s) URL or a path into the media store (`/api/media/...` or `/media/...`).
fn validate_media(text: &str) -> bool {
    validate_url(text)
        || (["/api/media/", "/media/"].iter().any(|prefix| {
            text.strip_prefix(prefix)
                .is_some_and(|rest| !rest.is_empty())
        }) && !text.chars().any(char::is_whitespace))
}

/// True for `lat,lon` coordinates in range, or any non-empty address text.
fn validate_place(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    if let Some((lat, lon)) = text.split_once(',') {
        if let (Ok(lat), Ok(lon)) = (lat.trim().parse::<f64>(), lon.trim().parse::<f64>()) {
            return (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lon);
        }
    }
    true
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
            op: AttributeFilterOp::try_from(value.op.as_str()).map_err(|e| e)?,
            value: value.value.clone(),
            list_mode: AttributeListMode::try_from(value.list_mode.as_str()).map_err(|e| e)?,
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
            "ilike" => Ok(Self::LikeCaseInsensitive),
            _ => Err(format!("{} operation not supported", value)),
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
            _ => Err(format!("{} list mode not supported", value)),
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
    fn errors_name_the_attribute_and_expected_shape() {
        let kinds = HashMap::from([(
            String::from("Parent"),
            kind(
                AttributeBaseType::List(AttributeBaseScalarType::Item),
                AttributeKindSpec::Item,
            ),
        )]);

        let err = Attribute::from_json(&json!({"Parent": 11}), &kinds).unwrap_err();
        assert_eq!(
            err.to_string(),
            "attribute 'Parent' (expects a JSON array of item id): value must be a JSON array"
        );

        let err = Attribute::from_json(&json!({"Mystery": [1]}), &kinds).unwrap_err();
        assert!(err.to_string().starts_with("attribute 'Mystery'"), "{err}");
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

    #[test]
    fn attribute_filter_dto_defaults_list_mode_to_any() {
        let dto: AttributeFilterDto =
            serde_json::from_value(json!({"key": "Status", "op": "eq", "value": "Open"})).unwrap();

        assert_eq!(dto.list_mode, "any");
    }

    #[test]
    fn parses_ilike_filter_operator() {
        let op = AttributeFilterOp::try_from("ilike").unwrap();

        assert!(matches!(op, AttributeFilterOp::LikeCaseInsensitive));
    }

    #[test]
    fn validates_formatted_kinds() {
        let cases = [
            (AttributeBaseScalarType::Email, json!("a@b.co"), true),
            (AttributeBaseScalarType::Email, json!("a@b"), false),
            (AttributeBaseScalarType::Email, json!("a b@c.com"), false),
            (AttributeBaseScalarType::Telephone, json!("+1 (555) 123-4567"), true),
            (AttributeBaseScalarType::Telephone, json!("12"), false),
            (AttributeBaseScalarType::Telephone, json!("555-abc-1234"), false),
            (AttributeBaseScalarType::Url, json!("https://example.com/x?y=1"), true),
            (AttributeBaseScalarType::Url, json!("example.com"), false),
            (AttributeBaseScalarType::Url, json!("javascript:alert(1)"), false),
            (AttributeBaseScalarType::Media, json!("/media/photo.png"), true),
            (AttributeBaseScalarType::Media, json!("/media/"), false),
            (AttributeBaseScalarType::Media, json!("/api/media/paste/x.png"), true),
            (AttributeBaseScalarType::Place, json!("10 Downing St, London"), true),
            (AttributeBaseScalarType::Place, json!("51.5, -0.12"), true),
            (AttributeBaseScalarType::Place, json!("95.0, 0"), false),
            (AttributeBaseScalarType::Place, json!(""), false),
            (AttributeBaseScalarType::Email, json!(5), false),
        ];
        for (scalar, value, ok) in cases {
            let k = kind(AttributeBaseType::Scalar(scalar.clone()), AttributeKindSpec::Email);
            assert_eq!(
                Attribute::single_from_json(&value, &k).is_ok(),
                ok,
                "{scalar:?} {value}"
            );
        }
    }

    #[test]
    fn multi_select_is_a_list_of_dropdown_values() {
        let k = kind(
            AttributeBaseType::List(AttributeBaseScalarType::Dropdown),
            AttributeKindSpec::Dropdown {
                values: vec!["a".into(), "b".into()],
            },
        );
        assert!(Attribute::single_from_json(&json!(["a", "b"]), &k).is_ok());
        assert!(Attribute::single_from_json(&json!(["a", "c"]), &k).is_err());
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
                AttributeScalar::Email(text)
                | AttributeScalar::Telephone(text)
                | AttributeScalar::Url(text)
                | AttributeScalar::Media(text)
                | AttributeScalar::Place(text) => serializer.serialize_str(text),
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
                        AttributeScalar::Email(text)
                        | AttributeScalar::Telephone(text)
                        | AttributeScalar::Url(text)
                        | AttributeScalar::Media(text)
                        | AttributeScalar::Place(text) => seq.serialize_element(text)?,
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
