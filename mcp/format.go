package main

import (
	"encoding/json"
	"fmt"
	"sort"
	"strings"
)

type CapabilityGroup struct {
	Router   string   `json:"router"`
	Routes   []string `json:"routes"`
	MCPTools []string `json:"mcp_tools"`
	Notes    string   `json:"notes,omitempty"`
}

var backendCapabilityMap = []CapabilityGroup{
	{
		Router: "account",
		Routes: []string{
			"GET /account/details",
			"GET /account/api-key",
			"PATCH /account/settings",
		},
		MCPTools: []string{
			"get_account_details",
			"get_api_key_status",
			"update_account_settings",
		},
		Notes: "Login/register/logout routes exist in the backend but are intentionally not exposed through MCP because the MCP already authenticates with an API key.",
	},
	{
		Router: "item",
		Routes: []string{
			"GET /item/",
			"GET /item/title/:title",
			"GET /item/id/:item_id",
			"GET /item/search",
			"GET /item/children/:item_id",
			"GET /item/related/:item_id",
			"POST /item/filter",
			"POST /item/",
			"PATCH /item/:item_id",
			"DELETE /item/:item_id",
			"PATCH /item/:item_id/attr",
			"PATCH /item/:item_id/attr/rename",
			"DELETE /item/:item_id/attr/:key",
			"POST /item/:item_id/assign_type/:type_name",
			"DELETE /item/:item_id/assign_type/:type_name",
		},
		MCPTools: []string{
			"get_today",
			"get_active_tickets",
			"get_item",
			"get_item_by_title",
			"search_items",
			"get_children",
			"get_related_items",
			"filter_items",
			"get_items_by_type",
			"create_item",
			"update_item",
			"delete_item",
			"set_item_attributes",
			"rename_item_attribute",
			"delete_item_attribute",
			"assign_item_type",
			"unassign_item_type",
		},
	},
	{
		Router: "item/type",
		Routes: []string{
			"GET /item/type/",
			"POST /item/type/",
			"POST /item/type/assign/:item_name",
			"DELETE /item/type/assign/:item_name",
			"PATCH /item/type/:type_id",
			"DELETE /item/type/:type_id",
		},
		MCPTools: []string{
			"list_item_types",
			"create_item_type",
			"update_item_type",
			"delete_item_type",
			"add_item_type_attribute_kinds",
			"remove_item_type_attribute_kinds",
		},
	},
	{
		Router: "item/kind",
		Routes: []string{
			"GET /item/kind/",
			"POST /item/kind/",
			"PATCH /item/kind/:kind_id",
			"PATCH /item/kind/:kind_id/config",
			"DELETE /item/kind/:kind_id",
		},
		MCPTools: []string{
			"list_attribute_kinds",
			"create_attribute_kind",
			"update_attribute_kind",
			"update_attribute_kind_config",
			"delete_attribute_kind",
		},
	},
	{
		Router: "planner",
		Routes: []string{
			"GET /planner/day/:date",
			"GET /planner/week/:week",
			"GET /planner/month/:month/year/:year",
			"GET /planner/year/:year",
		},
		MCPTools: []string{
			"get_planner_day",
			"get_planner_week",
			"get_planner_month",
			"get_planner_year",
		},
	},
	{
		Router: "repeat",
		Routes: []string{
			"GET /repeat/day/:date",
			"PATCH /repeat/:item_id/day/:date",
		},
		MCPTools: []string{
			"get_repeats_for_day",
			"set_repeat_status",
		},
	},
	{
		Router: "comments",
		Routes: []string{
			"GET /comments/day/:date",
			"GET /comments/item/:item_id",
			"POST /comments/",
			"PATCH /comments/:comment_id",
			"DELETE /comments/:comment_id",
		},
		MCPTools: []string{
			"get_comments_for_day",
			"get_comments_for_item",
			"add_comment",
			"update_comment",
			"delete_comment",
		},
	},
	{
		Router: "tracker",
		Routes: []string{
			"GET /tracker/day/:date",
			"POST /tracker/",
			"DELETE /tracker/:tracker_id",
		},
		MCPTools: []string{
			"get_tracker_day",
			"add_tracker_entry",
			"delete_tracker_entry",
		},
		Notes: "These routes exist in the backend and are now mounted by the server so the MCP can use them.",
	},
	{
		Router: "analysis",
		Routes: []string{
			"GET /analysis/last",
		},
		MCPTools: []string{
			"get_score_history",
		},
		Notes: "This route existed but was not mounted before; it is now available to the MCP.",
	},
	{
		Router: "media",
		Routes: []string{
			"GET /media/*",
			"POST /media/mkdir",
			"POST /media/*",
			"PATCH /media/rename",
			"DELETE /media/*",
		},
		MCPTools: []string{
			"list_media",
			"create_media_folder",
			"upload_media_file",
			"rename_media_entry",
			"delete_media_entry",
		},
		Notes: "The MCP maps the upload route to a local-path upload helper, so it can send files from the machine running the MCP server.",
	},
	{
		Router:   "rules",
		Routes:   []string{},
		MCPTools: []string{},
		Notes:    "rules_router.go currently defines no routes.",
	},
}

func formatCapabilityMap() string {
	return formatJSON(backendCapabilityMap)
}

func formatJSON(data any) string {
	if data == nil {
		return "No data."
	}

	pretty, err := json.MarshalIndent(data, "", "  ")
	if err != nil {
		return fmt.Sprintf("%+v", data)
	}
	return string(pretty)
}

func formatItems(items []Item) string {
	if len(items) == 0 {
		return "No items found."
	}

	var sb strings.Builder
	sb.WriteString(fmt.Sprintf("%d item(s):\n\n", len(items)))

	for i, item := range items {
		sb.WriteString(formatItem(&item))
		if i < len(items)-1 {
			sb.WriteString("\n")
		}
	}

	return sb.String()
}

func formatItem(item *Item) string {
	if item == nil {
		return "Item not found."
	}

	var sb strings.Builder
	sb.WriteString(fmt.Sprintf("## [%d] %s\n", item.ItemID, item.Title))

	if len(item.Types) > 0 {
		typeNames := make([]string, len(item.Types))
		for i, t := range item.Types {
			typeNames[i] = t.Name
		}
		sb.WriteString(fmt.Sprintf("Types: %s\n", strings.Join(typeNames, ", ")))
	}

	if strings.TrimSpace(item.Content) != "" {
		sb.WriteString(fmt.Sprintf("Content: %s\n", item.Content))
	}

	if len(item.Attributes) > 0 {
		keys := make([]string, 0, len(item.Attributes))
		for key := range item.Attributes {
			keys = append(keys, key)
		}
		sort.Strings(keys)

		sb.WriteString("Attributes:\n")
		for _, key := range keys {
			sb.WriteString(fmt.Sprintf("  %s: %v\n", key, item.Attributes[key]))
		}
	}

	return strings.TrimRight(sb.String(), "\n")
}
