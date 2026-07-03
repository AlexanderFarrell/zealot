use std::collections::{HashMap, HashSet};

use rmcp::{
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
    model::{GetPromptResult, PromptMessage, PromptMessageRole},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use zealot_domain::{
    item::{ItemDto, SearchResultDto, SearchScope},
    repeat::RepeatEntryDto,
};

use crate::{
    output::{self, Detail},
    tools::{ZealotServer, err_ctx, parse_date},
};

fn user_msg(text: String) -> PromptMessage {
    PromptMessage::new_text(PromptMessageRole::User, text)
}

fn json_block<T: Serialize>(v: &T) -> String {
    format!(
        "```json\n{}\n```",
        serde_json::to_string(v).unwrap_or_default()
    )
}

fn item_summaries(items: &[ItemDto]) -> Vec<output::ItemSummary> {
    items
        .iter()
        .map(|item| output::ItemSummary::project(item, Detail::Summary))
        .collect()
}

#[derive(Debug, Serialize)]
struct RepeatEntrySummary {
    item_id: i64,
    title: String,
    date: String,
    status: String,
    comment: String,
}

impl From<&RepeatEntryDto> for RepeatEntrySummary {
    fn from(entry: &RepeatEntryDto) -> Self {
        Self {
            item_id: entry.item.item_id,
            title: entry.item.title.clone(),
            date: entry.date.clone(),
            status: entry.status.clone(),
            comment: entry.comment.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
struct SearchPromptHit {
    #[serde(flatten)]
    item: output::ItemSummary,
    match_scope: SearchScope,
    snippet: Option<String>,
}

impl From<&SearchResultDto> for SearchPromptHit {
    fn from(hit: &SearchResultDto) -> Self {
        Self {
            item: output::ItemSummary::project(&hit.item, Detail::Summary),
            match_scope: hit.match_scope.clone(),
            snippet: hit.snippet.clone(),
        }
    }
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
zealot.get_item(id)                          -- Returns ItemDto or nil
zealot.search_items(term)                    -- Returns list of ItemDto
zealot.create_item(title, content)           -- Returns ItemDto
zealot.update_item(id, {title=, content=})   -- Returns ItemDto
zealot.set_attribute(item_id, key, value)    -- Sets an attribute
zealot.delete_attribute(item_id, key)        -- Deletes an attribute
zealot.assign_type(item_id, type_name)       -- Assigns a type
zealot.unassign_type(item_id, type_name)     -- Removes a type

-- Comments
zealot.add_comment(item_id, timestamp, content)  -- Returns CommentDto

-- Notifications & logging
zealot.notify(message)   -- Sends a notification (shown in UI)
zealot.log(message)      -- Logs to rule run output (visible in rule history)

-- Utilities
zealot.today()           -- Returns today's date as "YYYY-MM-DD"
zealot.now()             -- Returns current datetime as "YYYY-MM-DD HH:MM:SS"
```

### Example rule (notify on new Goal)
```lua
if ctx.item then
    zealot.notify("New Goal created: " .. ctx.item.title)
    zealot.set_attribute(ctx.item.item_id, "created_at", zealot.today())
end
```
"#;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DailyBriefingArgs {
    /// Date in YYYY-MM-DD format
    pub date: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PlanMyWeekArgs {
    /// ISO week string e.g. "2025-W20"
    pub week: String,
    /// Your goals or priorities for this week (free text)
    pub goals: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CaptureIdeaArgs {
    /// The idea to capture (free text)
    pub idea: String,
    /// Optional context or background for the idea
    pub context: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReflectOnGoalsArgs {
    /// Item type name to reflect on (e.g. "Goal", "Project")
    pub type_name: String,
    /// Optional time period context (e.g. "Q2 2025", "this month")
    pub period: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateProjectArgs {
    /// Project name
    pub name: String,
    /// Brief description of the project's purpose and outcome
    pub description: String,
    /// Comma-separated list of milestone names (optional)
    pub milestones: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReviewTasksArgs {
    /// Start date in YYYY-MM-DD format
    pub start_date: String,
    /// End date in YYYY-MM-DD format
    pub end_date: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchAndConnectArgs {
    /// Search query to explore
    pub query: String,
    /// How many hops of related items to follow (default 1, max 2)
    pub depth: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AutomateWorkflowArgs {
    /// Describe what should trigger the rule (e.g. "when a new item is created with type Goal")
    pub trigger_description: String,
    /// Describe what the rule should do (e.g. "send me a notification and add a due_date attribute 30 days out")
    pub action_description: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KnowledgeGraphArgs {
    /// Starting topic to explore
    pub topic: String,
    /// How many relationship hops to follow (default 2, max 3)
    pub max_depth: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExportSummaryArgs {
    /// Filter by item type name (optional)
    pub type_name: Option<String>,
    /// Search term to filter items (optional)
    pub search_term: Option<String>,
    /// Output format: "markdown" (default), "outline", or "table"
    pub format: Option<String>,
}

#[rmcp::prompt_router(vis = "pub")]
impl ZealotServer {
    #[rmcp::prompt(
        description = "Generate a concise daily briefing summarising today's scheduled items, habit status, and recent wiki activity."
    )]
    pub async fn daily_briefing(
        &self,
        Parameters(a): Parameters<DailyBriefingArgs>,
    ) -> Result<GetPromptResult, McpError> {
        let date = parse_date(&a.date)?;
        let (plan, repeats, recent) = tokio::join!(
            self.client.planner_day(date),
            self.client.repeats_for_day(date),
            self.client.recent_items(10, 0),
        );
        let plan = plan.map_err(|e| err_ctx("day plan", e))?;
        let repeats = repeats.map_err(|e| err_ctx("habit entries", e))?;
        let recent = recent.map_err(|e| err_ctx("recent items", e))?;
        let plan = item_summaries(&plan);
        let repeats: Vec<_> = repeats.iter().map(RepeatEntrySummary::from).collect();
        let recent = item_summaries(&recent);

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

    #[rmcp::prompt(
        description = "Given your goals for the week and what is already scheduled, suggest items to create, schedule, or prioritise."
    )]
    pub async fn plan_my_week(
        &self,
        Parameters(a): Parameters<PlanMyWeekArgs>,
    ) -> Result<GetPromptResult, McpError> {
        let week_plan = self
            .client
            .planner_week(&a.week)
            .await
            .map_err(|e| err_ctx("week plan", e))?;
        let week_plan = item_summaries(&week_plan);

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

    #[rmcp::prompt(
        description = "Turn a rough idea into a well-formed Zealot item: title, type, attributes, and content body."
    )]
    pub async fn capture_idea(
        &self,
        Parameters(a): Parameters<CaptureIdeaArgs>,
    ) -> Result<GetPromptResult, McpError> {
        let types = self
            .client
            .item_type_summaries()
            .await
            .map_err(|e| err_ctx("item types", e))?;

        let ctx = a
            .context
            .map(|c| format!("\n\n## Context\n{c}"))
            .unwrap_or_default();
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

    #[rmcp::prompt(
        description = "Review all items of a given type and provide a structured reflection on progress, blockers, and next steps."
    )]
    pub async fn reflect_on_goals(
        &self,
        Parameters(a): Parameters<ReflectOnGoalsArgs>,
    ) -> Result<GetPromptResult, McpError> {
        let items = self
            .client
            .list_items(Some(&a.type_name))
            .await
            .map_err(|e| err_ctx(&format!("items of type '{}'", a.type_name), e))?;
        let items = item_summaries(&items);

        let period_ctx = a
            .period
            .map(|p| format!(" for **{p}**"))
            .unwrap_or_default();
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

    #[rmcp::prompt(
        description = "Scaffold a complete project structure in Zealot: parent item, milestones, suggested attributes and types."
    )]
    pub async fn create_project(
        &self,
        Parameters(a): Parameters<CreateProjectArgs>,
    ) -> Result<GetPromptResult, McpError> {
        let (types, attributes) = tokio::join!(
            self.client.item_type_summaries(),
            self.client.list_attribute_kinds(),
        );
        let types = types.map_err(|e| err_ctx("item types", e))?;
        let attributes = attributes.map_err(|e| err_ctx("attribute kinds", e))?;

        let milestone_ctx = a
            .milestones
            .map(|m| {
                format!(
                    "\n\n## Requested milestones\n{}",
                    m.split(',')
                        .map(|s| format!("- {}", s.trim()))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })
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

    #[rmcp::prompt(
        description = "Review habit/repeat completion across a date range. Shows what was missed vs completed and suggests corrective actions."
    )]
    pub async fn review_tasks(
        &self,
        Parameters(a): Parameters<ReviewTasksArgs>,
    ) -> Result<GetPromptResult, McpError> {
        let start = parse_date(&a.start_date)?;
        let end = parse_date(&a.end_date)?;
        if end < start {
            return Err(McpError::invalid_params(
                "end_date must be on or after start_date",
                None,
            ));
        }
        if (end - start).num_days() > 365 {
            return Err(McpError::invalid_params(
                "review_tasks range is capped at 366 inclusive days",
                None,
            ));
        }

        let entries = self
            .client
            .repeats_for_range(start, end)
            .await
            .map_err(|e| err_ctx("habit entries", e))?;
        let entries: Vec<_> = entries.iter().map(RepeatEntrySummary::from).collect();

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
            data = json_block(&entries),
        );
        Ok(GetPromptResult::new(vec![user_msg(msg)]))
    }

    #[rmcp::prompt(
        description = "Search for items matching a query, explore their relationships, and suggest meaningful new connections to create."
    )]
    pub async fn search_and_connect(
        &self,
        Parameters(a): Parameters<SearchAndConnectArgs>,
    ) -> Result<GetPromptResult, McpError> {
        let depth = a.depth.unwrap_or(1).min(2);
        let results = self
            .client
            .search_items(&a.query, SearchScope::Title, false, 10, 0)
            .await
            .map_err(|e| err_ctx("search", e))?;
        let result_rows: Vec<_> = results.iter().map(SearchPromptHit::from).collect();

        let mut related_map: HashMap<i64, Vec<output::ItemSummary>> = HashMap::new();
        if depth >= 1 {
            for hit in results.iter().take(5) {
                let id = hit.item.item_id;
                let related = self
                    .client
                    .get_related(id)
                    .await
                    .map_err(|e| err_ctx(&format!("related items for item #{id}"), e))?;
                related_map.insert(id, item_summaries(&related));
            }
        }

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
            results = json_block(&result_rows),
            related = json_block(&related_map),
        );
        Ok(GetPromptResult::new(vec![user_msg(msg)]))
    }

    #[rmcp::prompt(
        description = "Write a Zealot automation rule: describe the trigger and action in plain English and get a complete, ready-to-use Lua script."
    )]
    pub async fn automate_workflow(
        &self,
        Parameters(a): Parameters<AutomateWorkflowArgs>,
    ) -> Result<GetPromptResult, McpError> {
        let (types, attributes) = tokio::join!(
            self.client.item_type_summaries(),
            self.client.list_attribute_kinds(),
        );
        let types = types.map_err(|e| err_ctx("item types", e))?;
        let attributes = attributes.map_err(|e| err_ctx("attribute kinds", e))?;

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

    #[rmcp::prompt(
        description = "Build a map of your knowledge graph starting from a topic item, following relationships up to max_depth hops."
    )]
    pub async fn knowledge_graph_explore(
        &self,
        Parameters(a): Parameters<KnowledgeGraphArgs>,
    ) -> Result<GetPromptResult, McpError> {
        let depth = a
            .max_depth
            .as_deref()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(2)
            .min(3);
        let root_results = self
            .client
            .search_items(&a.topic, SearchScope::Title, false, 10, 0)
            .await
            .map_err(|e| err_ctx("search", e))?;

        let mut visited: HashSet<i64> = HashSet::new();
        let mut graph: Vec<output::ItemSummary> = Vec::new();
        let mut frontier: Vec<i64> = Vec::new();

        for hit in root_results.iter().take(3) {
            if visited.insert(hit.item.item_id) {
                frontier.push(hit.item.item_id);
                graph.push(output::ItemSummary::project(&hit.item, Detail::Summary));
            }
        }

        for _ in 0..depth {
            let mut next = Vec::new();
            for id in &frontier {
                if graph.len() >= 20 {
                    break;
                }
                let children = self
                    .client
                    .get_children(*id)
                    .await
                    .map_err(|e| err_ctx(&format!("children of item #{id}"), e))?;
                for child in children.iter().take(5) {
                    if visited.insert(child.item_id) {
                        next.push(child.item_id);
                        graph.push(output::ItemSummary::project(child, Detail::Summary));
                    }
                }
                let related = self
                    .client
                    .get_related(*id)
                    .await
                    .map_err(|e| err_ctx(&format!("related items for item #{id}"), e))?;
                for rel in related.iter().take(5) {
                    if visited.insert(rel.item_id) {
                        next.push(rel.item_id);
                        graph.push(output::ItemSummary::project(rel, Detail::Summary));
                    }
                }
            }
            frontier = next;
            if frontier.is_empty() || graph.len() >= 20 {
                break;
            }
        }

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
            graph = json_block(&graph),
        );
        Ok(GetPromptResult::new(vec![user_msg(msg)]))
    }

    #[rmcp::prompt(
        description = "Generate a structured summary document of your Zealot items, filtered by type or search term, in your choice of format."
    )]
    pub async fn export_summary(
        &self,
        Parameters(a): Parameters<ExportSummaryArgs>,
    ) -> Result<GetPromptResult, McpError> {
        let items = if let Some(ref term) = a.search_term {
            let hits = self
                .client
                .search_items(term, SearchScope::Title, false, 50, 0)
                .await
                .map_err(|e| err_ctx("search", e))?;
            hits.iter()
                .map(|hit| output::ItemSummary::project(&hit.item, Detail::Summary))
                .collect()
        } else if let Some(ref type_name) = a.type_name {
            let items = self
                .client
                .list_items(Some(type_name))
                .await
                .map_err(|e| err_ctx(&format!("items of type '{type_name}'"), e))?;
            item_summaries(&items)
        } else {
            let items = self
                .client
                .recent_items(50, 0)
                .await
                .map_err(|e| err_ctx("recent items", e))?;
            item_summaries(&items)
        };

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
