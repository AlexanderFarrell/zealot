//! Keeps the MCP docs in step with the tools the server registers.
//!
//! `docs/mcp-system-prompt.md` is the agent-facing tool contract, so it must
//! name every registered tool, and neither MCP doc may name a tool the server
//! does not expose.

use std::collections::BTreeSet;

use serde_json::Value;
use zealot_mcp::tools::ZealotServer;

const SYSTEM_PROMPT_DOC: &str = include_str!("../../../docs/mcp-system-prompt.md");
const GUIDE_DOC: &str = include_str!("../../../docs/mcp.md");

/// Verbs that start tool names. A backticked snake_case token with one of
/// these prefixes is read as a tool reference unless it is a known parameter.
const TOOL_VERBS: &[&str] = &[
    "add",
    "append",
    "assign",
    "attribute",
    "browse",
    "create",
    "day",
    "delete",
    "edit",
    "filter",
    "get",
    "habit",
    "list",
    "orphaned",
    "rebuild",
    "record",
    "rename",
    "run",
    "search",
    "set",
    "unassign",
    "update",
    "upload",
    "wiki",
];

/// Names of every tool the server registers.
fn registered_tools() -> BTreeSet<String> {
    ZealotServer::tool_router()
        .list_all()
        .into_iter()
        .map(|tool| tool.name.to_string())
        .collect()
}

/// Every property name in every registered tool's input schema, including
/// nested definitions, so parameters such as `list_mode` are not mistaken for
/// tool names.
fn parameter_names() -> BTreeSet<String> {
    fn collect(value: &Value, names: &mut BTreeSet<String>) {
        match value {
            Value::Object(map) => {
                if let Some(Value::Object(properties)) = map.get("properties") {
                    names.extend(properties.keys().cloned());
                }
                map.values().for_each(|v| collect(v, names));
            }
            Value::Array(values) => values.iter().for_each(|v| collect(v, names)),
            _ => {}
        }
    }
    let mut names = BTreeSet::new();
    for tool in ZealotServer::tool_router().list_all() {
        collect(&Value::Object((*tool.input_schema).clone()), &mut names);
    }
    names
}

/// Tool names referenced in `doc`: the leading snake_case identifier of each
/// inline code span, kept when it is written as a call (`name(`) or starts with
/// a tool verb and is not a parameter name.
fn documented_tools(doc: &str, parameters: &BTreeSet<String>) -> BTreeSet<String> {
    doc.split('`')
        .skip(1)
        .step_by(2)
        .filter_map(|span| {
            let name: String = span
                .chars()
                .take_while(|c| c.is_ascii_lowercase() || *c == '_')
                .collect();
            if !name.contains('_') {
                return None;
            }
            let is_call = span[name.len()..].starts_with('(');
            let has_verb = TOOL_VERBS
                .iter()
                .any(|verb| name.starts_with(&format!("{verb}_")));
            (is_call || (has_verb && !parameters.contains(&name))).then_some(name)
        })
        .collect()
}

#[test]
fn documented_tools_are_registered() {
    let registered = registered_tools();
    let parameters = parameter_names();
    for (doc_name, doc) in [
        ("docs/mcp-system-prompt.md", SYSTEM_PROMPT_DOC),
        ("docs/mcp.md", GUIDE_DOC),
    ] {
        let missing: Vec<_> = documented_tools(doc, &parameters)
            .difference(&registered)
            .cloned()
            .collect();
        assert!(
            missing.is_empty(),
            "{doc_name} names tools the server does not register: {missing:?}"
        );
    }
}

#[test]
fn system_prompt_documents_every_registered_tool() {
    let documented = documented_tools(SYSTEM_PROMPT_DOC, &parameter_names());
    let undocumented: Vec<_> = registered_tools()
        .difference(&documented)
        .cloned()
        .collect();
    assert!(
        undocumented.is_empty(),
        "docs/mcp-system-prompt.md does not document registered tools: {undocumented:?}"
    );
}

#[test]
fn documented_tools_reads_calls_and_bare_names() {
    let parameters = BTreeSet::from(["list_mode".to_string()]);
    let doc = "`get_item(item)`, `delete_item`, `list_mode`, `on_item_create`, `ZEALOT_URL`, `daily_briefing`";
    assert_eq!(
        documented_tools(doc, &parameters),
        BTreeSet::from(["delete_item".to_string(), "get_item".to_string()])
    );
}
