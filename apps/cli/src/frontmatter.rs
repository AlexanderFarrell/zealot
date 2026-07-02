//! TOML frontmatter for `item edit --full`: title, types, and attributes are
//! edited in the same buffer as the content, delimited by `+++` lines.

use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::collections::HashMap;
use zealot_domain::item::ItemDto;

#[derive(Debug)]
pub struct FullItem {
    pub title: String,
    pub types: Vec<String>,
    pub attributes: HashMap<String, Value>,
    pub content: String,
}

pub fn to_buffer(item: &ItemDto) -> String {
    let mut doc = toml::value::Table::new();
    doc.insert("title".into(), toml::Value::String(item.title.clone()));
    doc.insert(
        "types".into(),
        toml::Value::Array(
            item.types
                .iter()
                .map(|t| toml::Value::String(t.name.clone()))
                .collect(),
        ),
    );
    let mut attrs = toml::value::Table::new();
    if let Value::Object(map) = &item.attributes {
        for (k, v) in map {
            attrs.insert(k.clone(), json_to_toml(v));
        }
    }
    doc.insert("attributes".into(), toml::Value::Table(attrs));

    format!(
        "+++\n{}+++\n{}",
        toml::to_string_pretty(&doc).unwrap_or_default(),
        item.content
    )
}

pub fn parse_buffer(text: &str) -> Result<FullItem> {
    let rest = text
        .strip_prefix("+++\n")
        .context("missing opening '+++' frontmatter delimiter")?;
    let end = rest
        .find("\n+++")
        .context("missing closing '+++' frontmatter delimiter")?;
    let (front, tail) = rest.split_at(end);
    let content = tail
        .strip_prefix("\n+++")
        .unwrap_or(tail)
        .strip_prefix('\n')
        .unwrap_or("")
        .to_string();

    let doc: toml::value::Table = toml::from_str(front).context("invalid TOML frontmatter")?;
    let title = match doc.get("title") {
        Some(toml::Value::String(s)) if !s.trim().is_empty() => s.clone(),
        _ => bail!("frontmatter needs a non-empty string 'title'"),
    };
    let types = match doc.get("types") {
        Some(toml::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        None => Vec::new(),
        _ => bail!("'types' must be an array of strings"),
    };
    let attributes = match doc.get("attributes") {
        Some(toml::Value::Table(table)) => table
            .iter()
            .map(|(k, v)| (k.clone(), toml_to_json(v)))
            .collect(),
        None => HashMap::new(),
        _ => bail!("'attributes' must be a table"),
    };

    Ok(FullItem {
        title,
        types,
        attributes,
        content,
    })
}

fn json_to_toml(v: &Value) -> toml::Value {
    match v {
        Value::String(s) => toml::Value::String(s.clone()),
        Value::Bool(b) => toml::Value::Boolean(*b),
        Value::Number(n) if n.is_i64() => toml::Value::Integer(n.as_i64().unwrap()),
        Value::Number(n) => toml::Value::Float(n.as_f64().unwrap_or(0.0)),
        Value::Array(arr) => toml::Value::Array(arr.iter().map(json_to_toml).collect()),
        other => toml::Value::String(other.to_string()),
    }
}

fn toml_to_json(v: &toml::Value) -> Value {
    match v {
        toml::Value::String(s) => Value::String(s.clone()),
        toml::Value::Integer(i) => Value::from(*i),
        toml::Value::Float(f) => Value::from(*f),
        toml::Value::Boolean(b) => Value::Bool(*b),
        toml::Value::Array(arr) => Value::Array(arr.iter().map(toml_to_json).collect()),
        toml::Value::Datetime(dt) => Value::String(dt.to_string()),
        toml::Value::Table(t) => Value::Object(
            t.iter()
                .map(|(k, v)| (k.clone(), toml_to_json(v)))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_an_item() {
        let item = ItemDto {
            item_id: 1,
            title: "Note".into(),
            content: "# Hello\nBody".into(),
            attributes: serde_json::json!({"Status": "Open", "Priority": 2}),
            types: vec![],
            links: vec![],
        };
        let buffer = to_buffer(&item);
        let parsed = parse_buffer(&buffer).unwrap();
        assert_eq!(parsed.title, "Note");
        assert_eq!(parsed.content, "# Hello\nBody");
        assert_eq!(parsed.attributes["Status"], serde_json::json!("Open"));
        assert_eq!(parsed.attributes["Priority"], serde_json::json!(2));
    }

    #[test]
    fn rejects_missing_title() {
        let err = parse_buffer("+++\ntypes = []\n+++\nbody").unwrap_err();
        assert!(err.to_string().contains("title"));
    }
}
