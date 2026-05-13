# MCP-006: Prompts

## Goal

Implement 10 powerful MCP prompt templates in `apps/mcp/src/prompts/mod.rs`. Each prompt pre-fetches live Zealot data and weaves it into a rich user message that makes the LLM's response immediately grounded and actionable.

## Depends On

MCP-001 through MCP-005 (prompts call the same `ZealotClient` methods as the tools)

## Design Principles

- **Pre-fetch, don't delegate**: each prompt makes API calls at `prompts/get` time so the LLM gets real data, not instructions to call tools first.
- **User role messages only**: all content is returned as `PromptMessageRole::User` — the LLM has no pre-canned assistant persona.
- **Structured data as fenced code blocks**: Zealot API responses are embedded as ` ```json ` blocks so the LLM can parse and reason about them.
- **Clear closing instruction**: each message ends with a natural-language directive telling the LLM what to do with the data.

## File: `apps/mcp/src/prompts/mod.rs`

### Imports and shared setup

```rust
use rmcp::{
    prompt,
    model::{GetPromptResult, PromptMessage, PromptMessageRole, TextContent},
    Error as McpError,
    handler::server::tool::Parameters,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::tools::ZealotServer;

fn user_msg(text: String) -> PromptMessage {
    PromptMessage {
        role: PromptMessageRole::User,
        content: rmcp::model::PromptMessageContent::Text(TextContent { text, annotations: None }),
    }
}

fn json_block(v: &serde_json::Value) -> String {
    format!("```json\n{}\n```", serde_json::to_string_pretty(v).unwrap_or_default())
}
```

### Prompt 1: `daily_briefing`

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DailyBriefingArgs {
    /// Date in YYYY-MM-DD format
    pub date: String,
}

impl ZealotServer {
    #[prompt(description = "Generate a concise daily briefing summarising today's scheduled items, habit status, and recent wiki activity.")]
    pub async fn daily_briefing(&self, Parameters(a): Parameters<DailyBriefingArgs>) -> Result<GetPromptResult, McpError> {
        let (plan, repeats, recent) = tokio::try_join!(
            self.client.get::<serde_json::Value>(&format!("/planner/day/{}", a.date)),
            self.client.get::<serde_json::Value>(&format!("/repeat/day/{}", a.date)),
            self.client.get::<serde_json::Value>("/item/recent?limit=10&offset=0"),
        ).map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let msg = format!(
            "Please give me a daily briefing for **{date}**.\n\n\
            ## Scheduled items for today\n{plan}\n\n\
            ## Habit / repeat entries\n{repeats}\n\n\
            ## Recently active wiki items\n{recent}\n\n\
            Summarise what I need to accomplish today. Call out any habits that haven't been marked complete. \
            Highlight any recent wiki items that are likely relevant to today's work.",
            date = a.date,
            plan = json_block(&plan),
            repeats = json_block(&repeats),
            recent = json_block(&recent),
        );
        Ok(GetPromptResult::new(vec![user_msg(msg)]))
    }
}
```

### Prompt 2: `plan_my_week`

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PlanMyWeekArgs {
    /// ISO week string e.g. "2025-W20"
    pub week: String,
    /// Your goals or priorities for this week (free text)
    pub goals: String,
}

impl ZealotServer {
    #[prompt(description = "Given your goals for the week and what is already scheduled, suggest items to create, schedule, or prioritise.")]
    pub async fn plan_my_week(&self, Parameters(a): Parameters<PlanMyWeekArgs>) -> Result<GetPromptResult, McpError> {
        let week_plan = self.client.get::<serde_json::Value>(&format!("/planner/week/{}", a.week))
            .await.unwrap_or(serde_json::Value::Array(vec![]));

        let msg = format!(
            "Help me plan week **{week}**.\n\n\
            ## My goals for this week\n{goals}\n\n\
            ## Items already scheduled this week\n{plan}\n\n\
            Based on my goals, suggest:\n\
            1. New items I should create (with suggested titles, types, and content outlines)\n\
            2. Existing items I should focus on or reprioritise\n\
            3. Any habits I should pay extra attention to\n\
            Be specific and actionable.",
            week = a.week,
            goals = a.goals,
            plan = json_block(&week_plan),
        );
        Ok(GetPromptResult::new(vec![user_msg(msg)]))
    }
}
```

### Prompt 3: `capture_idea`

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CaptureIdeaArgs {
    /// The idea to capture (free text)
    pub idea: String,
    /// Optional context or background for the idea
    pub context: Option<String>,
}

impl ZealotServer {
    #[prompt(description = "Turn a rough idea into a well-formed Zealot item: title, type, attributes, and content body.")]
    pub async fn capture_idea(&self, Parameters(a): Parameters<CaptureIdeaArgs>) -> Result<GetPromptResult, McpError> {
        let types = self.client.get::<serde_json::Value>("/item_type/summary")
            .await.unwrap_or(serde_json::Value::Array(vec![]));

        let ctx = a.context.map(|c| format!("\n\n## Context\n{c}")).unwrap_or_default();
        let msg = format!(
            "I have a new idea I want to capture in my Zealot wiki:\n\n> {idea}{ctx}\n\n\
            ## Available item types\n{types}\n\n\
            Please help me capture this as a well-structured Zealot item. Provide:\n\
            1. A clear, concise **title**\n\
            2. The most appropriate **type** from the available types (or suggest a new one if none fit)\n\
            3. Suggested **attributes** relevant to this type of item\n\
            4. A **content body** in markdown — include a summary, key considerations, and next actions\n\n\
            Format your response so I can copy the fields directly into Zealot.",
            idea = a.idea,
            ctx = ctx,
            types = json_block(&types),
        );
        Ok(GetPromptResult::new(vec![user_msg(msg)]))
    }
}
```

### Prompt 4: `reflect_on_goals`

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReflectOnGoalsArgs {
    /// Item type name to reflect on (e.g. "Goal", "Project")
    pub type_name: String,
    /// Optional time period context (e.g. "Q2 2025", "this month")
    pub period: Option<String>,
}

impl ZealotServer {
    #[prompt(description = "Review all items of a given type and provide a structured reflection on progress, blockers, and next steps.")]
    pub async fn reflect_on_goals(&self, Parameters(a): Parameters<ReflectOnGoalsArgs>) -> Result<GetPromptResult, McpError> {
        let items = self.client.get::<serde_json::Value>(&format!("/item/?type={}", urlencoding::encode(&a.type_name)))
            .await.unwrap_or(serde_json::Value::Array(vec![]));

        let period_ctx = a.period.map(|p| format!(" for **{p}**")).unwrap_or_default();
        let msg = format!(
            "Please help me reflect on my **{type_name}** items{period_ctx}.\n\n\
            ## Current {type_name} items\n{items}\n\n\
            For each item, assess:\n\
            - What progress has been made?\n\
            - What blockers or risks exist?\n\
            - What is the most important next action?\n\n\
            Then give me an overall summary: what's going well, what needs attention, \
            and what I should consider dropping or deprioritising.",
            type_name = a.type_name,
            period_ctx = period_ctx,
            items = json_block(&items),
        );
        Ok(GetPromptResult::new(vec![user_msg(msg)]))
    }
}
```

### Prompt 5: `create_project`

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateProjectArgs {
    /// Project name
    pub name: String,
    /// Brief description of the project's purpose and outcome
    pub description: String,
    /// Comma-separated list of milestone names (optional)
    pub milestones: Option<String>,
}

impl ZealotServer {
    #[prompt(description = "Scaffold a complete project structure in Zealot: parent item, milestones, suggested attributes and types.")]
    pub async fn create_project(&self, Parameters(a): Parameters<CreateProjectArgs>) -> Result<GetPromptResult, McpError> {
        let types = self.client.get::<serde_json::Value>("/item_type/summary")
            .await.unwrap_or(serde_json::Value::Array(vec![]));
        let attributes = self.client.get::<serde_json::Value>("/attribute/")
            .await.unwrap_or(serde_json::Value::Array(vec![]));

        let milestone_ctx = a.milestones
            .map(|m| format!("\n\n## Requested milestones\n{}", m.split(',').map(|s| format!("- {}", s.trim())).collect::<Vec<_>>().join("\n")))
            .unwrap_or_default();

        let msg = format!(
            "Help me scaffold a new project in Zealot.\n\n\
            **Project name:** {name}\n\
            **Description:** {description}{milestones}\n\n\
            ## Available item types\n{types}\n\n\
            ## Available attribute kinds\n{attributes}\n\n\
            Please generate a complete project scaffold:\n\
            1. The **parent project item** — title, type, content, key attributes\n\
            2. **Child items** for each milestone/phase (with titles and brief content)\n\
            3. Suggested **attributes** to track on the project (due date, priority, status, etc.)\n\
            4. Any **sub-tasks** worth creating immediately\n\n\
            Output the structure as a numbered list of `create_item` calls I should make, \
            with parent/child relationships clearly noted.",
            name = a.name,
            description = a.description,
            milestones = milestone_ctx,
            types = json_block(&types),
            attributes = json_block(&attributes),
        );
        Ok(GetPromptResult::new(vec![user_msg(msg)]))
    }
}
```

### Prompt 6: `review_tasks`

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReviewTasksArgs {
    /// Start date in YYYY-MM-DD format
    pub start_date: String,
    /// End date in YYYY-MM-DD format
    pub end_date: String,
}

impl ZealotServer {
    #[prompt(description = "Review habit/repeat completion across a date range. Shows what was missed vs completed and suggests corrective actions.")]
    pub async fn review_tasks(&self, Parameters(a): Parameters<ReviewTasksArgs>) -> Result<GetPromptResult, McpError> {
        // Fetch repeat entries for each day in range (up to 14 days to avoid overwhelming the context)
        use chrono::NaiveDate;
        let start = NaiveDate::parse_from_str(&a.start_date, "%Y-%m-%d")
            .map_err(|_| McpError::invalid_params("invalid start_date format (expected YYYY-MM-DD)", None))?;
        let end = NaiveDate::parse_from_str(&a.end_date, "%Y-%m-%d")
            .map_err(|_| McpError::invalid_params("invalid end_date format (expected YYYY-MM-DD)", None))?;

        let days = (end - start).num_days().min(14);
        let mut all_entries: Vec<serde_json::Value> = vec![];
        for i in 0..=days {
            let date = (start + chrono::Duration::days(i)).format("%Y-%m-%d").to_string();
            if let Ok(entries) = self.client.get::<serde_json::Value>(&format!("/repeat/day/{}", date)).await {
                if let Some(arr) = entries.as_array() {
                    all_entries.extend(arr.clone());
                }
            }
        }

        let summary = serde_json::Value::Array(all_entries);
        let msg = format!(
            "Please review my habit and task completion from **{start}** to **{end}**.\n\n\
            ## Repeat entries for this period\n{data}\n\n\
            Provide:\n\
            1. A completion rate summary for each habit/task\n\
            2. Which days had the most misses\n\
            3. Any patterns worth noting (e.g. consistently skipped on weekends)\n\
            4. Concrete suggestions for improving consistency or adjusting expectations",
            start = a.start_date,
            end = a.end_date,
            data = json_block(&summary),
        );
        Ok(GetPromptResult::new(vec![user_msg(msg)]))
    }
}
```

### Prompt 7: `search_and_connect`

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchAndConnectArgs {
    /// Search query to explore
    pub query: String,
    /// How many hops of related items to follow (default 1, max 2)
    pub depth: Option<u32>,
}

impl ZealotServer {
    #[prompt(description = "Search for items matching a query, explore their relationships, and suggest meaningful new connections to create.")]
    pub async fn search_and_connect(&self, Parameters(a): Parameters<SearchAndConnectArgs>) -> Result<GetPromptResult, McpError> {
        let depth = a.depth.unwrap_or(1).min(2);
        let results = self.client.get::<serde_json::Value>(&format!("/item/search?term={}", urlencoding::encode(&a.query)))
            .await.unwrap_or(serde_json::Value::Array(vec![]));

        let mut related_map: std::collections::HashMap<i64, serde_json::Value> = Default::default();
        if depth >= 1 {
            if let Some(arr) = results.as_array() {
                for item in arr.iter().take(5) {
                    if let Some(id) = item.get("item_id").and_then(|v| v.as_i64()) {
                        if let Ok(rel) = self.client.get::<serde_json::Value>(&format!("/item/related/{id}")).await {
                            related_map.insert(id, rel);
                        }
                    }
                }
            }
        }

        let related_json = serde_json::to_value(&related_map).unwrap_or_default();
        let msg = format!(
            "Help me explore and connect knowledge around the topic: **{query}**\n\n\
            ## Search results\n{results}\n\n\
            ## Related items (1 hop)\n{related}\n\n\
            Please:\n\
            1. Describe the knowledge graph I have around this topic\n\
            2. Identify gaps — important concepts or connections that are missing\n\
            3. Suggest 3–5 specific new relationships or links I should add between existing items\n\
            4. Suggest any new items I should create to fill the gaps",
            query = a.query,
            results = json_block(&results),
            related = json_block(&related_json),
        );
        Ok(GetPromptResult::new(vec![user_msg(msg)]))
    }
}
```

### Prompt 8: `automate_workflow`

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AutomateWorkflowArgs {
    /// Describe what should trigger the rule (e.g. "when a new item is created with type Goal")
    pub trigger_description: String,
    /// Describe what the rule should do (e.g. "send me a notification and add a due_date attribute 30 days out")
    pub action_description: String,
}

const LUA_API_REFERENCE: &str = r#"
## Zealot Lua Rule API

The script runs inside a Lua 5.4 sandbox with access to the `zealot` global table.

### Context variables (injected at runtime)
- `ctx.item` — the item that triggered the rule (ItemDto), or nil
- `ctx.comment` — the comment that triggered the rule (CommentDto), or nil
- `ctx.type_name` — the type name for type-assign triggers, or nil
- `ctx.attribute_key` — the attribute key for attribute-set triggers, or nil

### zealot.* API
```lua
-- Items
zealot.get_item(id)                   -- Returns ItemDto or nil
zealot.search_items(term)             -- Returns list of ItemDto
zealot.create_item(title, content)    -- Returns ItemDto
zealot.update_item(id, {title=, content=}) -- Returns ItemDto
zealot.set_attribute(item_id, key, value)  -- Sets an attribute
zealot.delete_attribute(item_id, key)      -- Deletes an attribute
zealot.assign_type(item_id, type_name)     -- Assigns a type
zealot.unassign_type(item_id, type_name)   -- Removes a type

-- Comments
zealot.add_comment(item_id, timestamp, content) -- Returns CommentDto

-- Notifications
zealot.notify(message)  -- Sends a notification (displayed in UI)
zealot.log(message)     -- Logs to rule run output (visible in rule history)

-- Utilities
zealot.today()          -- Returns today's date as "YYYY-MM-DD"
zealot.now()            -- Returns current datetime as "YYYY-MM-DD HH:MM:SS"
```

### Example rule (notify on new Goal)
```lua
if ctx.item then
    zealot.notify("New Goal created: " .. ctx.item.title)
    zealot.set_attribute(ctx.item.item_id, "created_at", zealot.today())
end
```
"#;

impl ZealotServer {
    #[prompt(description = "Write a Zealot automation rule: describe the trigger and action in plain English and get a complete, ready-to-use Lua script.")]
    pub async fn automate_workflow(&self, Parameters(a): Parameters<AutomateWorkflowArgs>) -> Result<GetPromptResult, McpError> {
        let types = self.client.get::<serde_json::Value>("/item_type/summary")
            .await.unwrap_or(serde_json::Value::Array(vec![]));
        let attributes = self.client.get::<serde_json::Value>("/attribute/")
            .await.unwrap_or(serde_json::Value::Array(vec![]));

        let msg = format!(
            "Help me write a Zealot automation rule.\n\n\
            **Trigger:** {trigger}\n\
            **Action:** {action}\n\n\
            ## My item types\n{types}\n\n\
            ## My attribute kinds\n{attributes}\n\n\
            {lua_ref}\n\
            Please produce:\n\
            1. The correct **trigger JSON** (e.g. `{{\"kind\":\"on_type_assign\",\"type_name\":\"Goal\"}}`)\n\
            2. A complete, working **Lua script** that implements the action\n\
            3. A brief explanation of what the rule does and any edge cases to watch for\n\n\
            Make the script production-ready with appropriate `zealot.log()` calls for debugging.",
            trigger = a.trigger_description,
            action = a.action_description,
            types = json_block(&types),
            attributes = json_block(&attributes),
            lua_ref = LUA_API_REFERENCE,
        );
        Ok(GetPromptResult::new(vec![user_msg(msg)]))
    }
}
```

### Prompt 9: `knowledge_graph_explore`

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct KnowledgeGraphArgs {
    /// Starting topic to explore
    pub topic: String,
    /// How many relationship hops to follow (default 2, max 3)
    pub max_depth: Option<u32>,
}

impl ZealotServer {
    #[prompt(description = "Build a map of your knowledge graph starting from a topic item, following relationships up to max_depth hops.")]
    pub async fn knowledge_graph_explore(&self, Parameters(a): Parameters<KnowledgeGraphArgs>) -> Result<GetPromptResult, McpError> {
        let depth = a.max_depth.unwrap_or(2).min(3);
        let root_results = self.client.get::<serde_json::Value>(&format!("/item/search?term={}", urlencoding::encode(&a.topic)))
            .await.unwrap_or(serde_json::Value::Array(vec![]));

        // BFS up to depth, collecting up to 20 items total
        let mut visited: std::collections::HashSet<i64> = Default::default();
        let mut graph: Vec<serde_json::Value> = vec![];
        let mut frontier: Vec<i64> = vec![];

        if let Some(arr) = root_results.as_array() {
            for item in arr.iter().take(3) {
                if let Some(id) = item.get("item_id").and_then(|v| v.as_i64()) {
                    frontier.push(id);
                    visited.insert(id);
                    graph.push(item.clone());
                }
            }
        }

        for _ in 0..depth {
            let mut next: Vec<i64> = vec![];
            for id in &frontier {
                if graph.len() >= 20 { break; }
                if let Ok(children) = self.client.get::<serde_json::Value>(&format!("/item/children/{id}")).await {
                    if let Some(arr) = children.as_array() {
                        for child in arr.iter().take(5) {
                            if let Some(cid) = child.get("item_id").and_then(|v| v.as_i64()) {
                                if visited.insert(cid) { next.push(cid); graph.push(child.clone()); }
                            }
                        }
                    }
                }
                if let Ok(related) = self.client.get::<serde_json::Value>(&format!("/item/related/{id}")).await {
                    if let Some(arr) = related.as_array() {
                        for rel in arr.iter().take(5) {
                            if let Some(rid) = rel.get("item_id").and_then(|v| v.as_i64()) {
                                if visited.insert(rid) { next.push(rid); graph.push(rel.clone()); }
                            }
                        }
                    }
                }
            }
            frontier = next;
        }

        let graph_val = serde_json::Value::Array(graph);
        let msg = format!(
            "Explore my knowledge graph around the topic: **{topic}**\n\n\
            ## Discovered items ({depth} hops from topic)\n{graph}\n\n\
            Please:\n\
            1. Describe the overall structure and themes in this part of my wiki\n\
            2. Identify the most central / important items\n\
            3. Point out any isolated items that should be connected\n\
            4. Suggest 3–5 new items or connections that would strengthen this knowledge cluster",
            topic = a.topic,
            depth = depth,
            graph = json_block(&graph_val),
        );
        Ok(GetPromptResult::new(vec![user_msg(msg)]))
    }
}
```

### Prompt 10: `export_summary`

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExportSummaryArgs {
    /// Filter by item type name (optional)
    pub type_name: Option<String>,
    /// Search term to filter items (optional)
    pub search_term: Option<String>,
    /// Output format: "markdown" (default), "outline", or "table"
    pub format: Option<String>,
}

impl ZealotServer {
    #[prompt(description = "Generate a structured summary document of your Zealot items, filtered by type or search term, in your choice of format.")]
    pub async fn export_summary(&self, Parameters(a): Parameters<ExportSummaryArgs>) -> Result<GetPromptResult, McpError> {
        let items = if let Some(ref term) = a.search_term {
            self.client.get::<serde_json::Value>(&format!("/item/search?term={}", urlencoding::encode(term))).await
        } else if let Some(ref type_name) = a.type_name {
            self.client.get::<serde_json::Value>(&format!("/item/?type={}", urlencoding::encode(type_name))).await
        } else {
            self.client.get::<serde_json::Value>("/item/recent?limit=50&offset=0").await
        }.unwrap_or(serde_json::Value::Array(vec![]));

        let format = a.format.as_deref().unwrap_or("markdown");
        let filter_desc = match (&a.type_name, &a.search_term) {
            (Some(t), _) => format!("type: **{t}**"),
            (_, Some(s)) => format!("matching: **\"{s}\"**"),
            _ => "50 most recent items".to_string(),
        };

        let msg = format!(
            "Generate a **{format}** summary document of my Zealot items ({filter}).\n\n\
            ## Items to summarise\n{items}\n\n\
            Format requirements:\n\
            - **markdown**: headings, bullet points, include key attributes and a one-line summary per item\n\
            - **outline**: indented nested outline, parent items first, children indented below\n\
            - **table**: markdown table with columns: Title | Type | Key Attributes | Summary\n\n\
            Make it human-readable and suitable for sharing or exporting. \
            If there are too many items to cover in detail, group similar ones and summarise the group.",
            format = format,
            filter = filter_desc,
            items = json_block(&items),
        );
        Ok(GetPromptResult::new(vec![user_msg(msg)]))
    }
}
```

## Wiring up the prompt router

In `tools/mod.rs`, the `ServerHandler` impl needs `#[prompt_handler]`. The `prompts/mod.rs` module must be included in `main.rs` as `mod prompts;` and all prompt methods collected by `rmcp`'s macro system.

## Add `chrono` dependency

In `apps/mcp/Cargo.toml`:

```toml
chrono.workspace = true
```

(Already a workspace dep, so `.workspace = true` is enough.)

## Verify

```bash
cargo check -p zealot-mcp

# List all prompts:
echo '{"jsonrpc":"2.0","id":1,"method":"prompts/list","params":{}}' | \
  ZEALOT_API_KEY=mykey cargo run -p zealot-mcp 2>/dev/null | jq '.result.prompts | map(.name)'
```

Expected: 10 prompt names in the output.
