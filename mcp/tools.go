package main

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/mark3labs/mcp-go/mcp"
	"github.com/mark3labs/mcp-go/server"
)

func invalidArguments(err error) (*mcp.CallToolResult, error) {
	return mcp.NewToolResultError(fmt.Sprintf("invalid arguments: %v", err)), nil
}

func toolFailure(action string, err error) (*mcp.CallToolResult, error) {
	return mcp.NewToolResultError(fmt.Sprintf("failed to %s: %v", action, err)), nil
}

func registerTools(s *server.MCPServer, client *ZealotClient) {
	registerCapabilityTools(s)
	registerAccountTools(s, client)
	registerItemTools(s, client)
	registerMetadataTools(s, client)
	registerPlannerTools(s, client)
	registerRepeatTools(s, client)
	registerCommentTools(s, client)
	registerTrackerTools(s, client)
	registerMediaTools(s, client)
}

func registerCapabilityTools(s *server.MCPServer) {
	s.AddTool(mcp.NewTool("describe_backend_capabilities",
		mcp.WithDescription("Describe the backend capability map discovered from the Zealot router files and the MCP tools that cover each area."),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		return mcp.NewToolResultText(formatCapabilityMap()), nil
	})
}

func registerAccountTools(s *server.MCPServer, client *ZealotClient) {
	s.AddTool(mcp.NewTool("get_account_details",
		mcp.WithDescription("Get the current account profile and raw settings JSON for the API key used by the MCP server."),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		details, err := client.GetAccountDetails()
		if err != nil {
			return toolFailure("get account details", err)
		}
		return mcp.NewToolResultText(formatJSON(details)), nil
	})

	s.AddTool(mcp.NewTool("get_api_key_status",
		mcp.WithDescription("Check whether the current account has an API key registered."),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		status, err := client.GetAPIKeyStatus()
		if err != nil {
			return toolFailure("get API key status", err)
		}
		return mcp.NewToolResultText(formatJSON(status)), nil
	})

	s.AddTool(mcp.NewTool("update_account_settings",
		mcp.WithDescription("Replace account settings with the provided JSON object."),
		mcp.WithObject("settings", mcp.Required(), mcp.Description("The full settings JSON object to save.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		settings, err := requiredArg[json.RawMessage](req, "settings")
		if err != nil {
			return invalidArguments(err)
		}
		if err := client.UpdateAccountSettings(settings); err != nil {
			return toolFailure("update account settings", err)
		}
		return mcp.NewToolResultText("Account settings updated."), nil
	})
}

func registerItemTools(s *server.MCPServer, client *ZealotClient) {
	s.AddTool(mcp.NewTool("get_today",
		mcp.WithDescription("Get today's planner items including tasks, habits, and scored daily entries."),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		items, err := client.GetToday()
		if err != nil {
			return toolFailure("get today's items", err)
		}
		return mcp.NewToolResultText(formatItems(items)), nil
	})

	s.AddTool(mcp.NewTool("get_active_tickets",
		mcp.WithDescription("Get all items currently in Working status."),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		items, err := client.GetActiveTickets()
		if err != nil {
			return toolFailure("get active tickets", err)
		}
		return mcp.NewToolResultText(formatItems(items)), nil
	})

	s.AddTool(mcp.NewTool("get_item",
		mcp.WithDescription("Get a specific Zealot item by numeric ID."),
		mcp.WithNumber("item_id", mcp.Required(), mcp.Description("The numeric item ID.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		itemID, err := requiredArg[int](req, "item_id")
		if err != nil {
			return invalidArguments(err)
		}

		item, err := client.GetItemByID(itemID)
		if err != nil {
			return toolFailure(fmt.Sprintf("get item %d", itemID), err)
		}
		return mcp.NewToolResultText(formatItem(item)), nil
	})

	s.AddTool(mcp.NewTool("get_item_by_title",
		mcp.WithDescription("Get an item by exact title."),
		mcp.WithString("title", mcp.Required(), mcp.Description("The exact item title.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		title, err := requiredNonEmptyString(req, "title")
		if err != nil {
			return invalidArguments(err)
		}

		item, err := client.GetItemByTitle(title)
		if err != nil {
			return toolFailure("get item by title", err)
		}
		return mcp.NewToolResultText(formatItem(item)), nil
	})

	s.AddTool(mcp.NewTool("search_items",
		mcp.WithDescription("Search Zealot items by title using a case-insensitive partial match."),
		mcp.WithString("query", mcp.Required(), mcp.Description("Title search term.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		query, err := requiredNonEmptyString(req, "query")
		if err != nil {
			return invalidArguments(err)
		}

		items, err := client.SearchItems(query)
		if err != nil {
			return toolFailure("search items", err)
		}
		return mcp.NewToolResultText(formatItems(items)), nil
	})

	s.AddTool(mcp.NewTool("get_children",
		mcp.WithDescription("Get all child items whose Parent attribute points to the given item."),
		mcp.WithNumber("item_id", mcp.Required(), mcp.Description("The parent item ID.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		itemID, err := requiredArg[int](req, "item_id")
		if err != nil {
			return invalidArguments(err)
		}

		items, err := client.GetChildren(itemID)
		if err != nil {
			return toolFailure("get child items", err)
		}
		return mcp.NewToolResultText(formatItems(items)), nil
	})

	s.AddTool(mcp.NewTool("get_related_items",
		mcp.WithDescription("Get items related to a given item by Zealot's related-item lookup."),
		mcp.WithNumber("item_id", mcp.Required(), mcp.Description("The source item ID.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		itemID, err := requiredArg[int](req, "item_id")
		if err != nil {
			return invalidArguments(err)
		}

		items, err := client.GetRelatedItems(itemID)
		if err != nil {
			return toolFailure("get related items", err)
		}
		return mcp.NewToolResultText(formatItems(items)), nil
	})

	s.AddTool(mcp.NewTool("filter_items",
		mcp.WithDescription(`Filter Zealot items by attribute conditions.
Each filter uses {key, op, value, list_mode} where op is one of eq, ne, gt, gte, lt, lte, ilike and list_mode is any, all, or none.`),
		mcp.WithArray(
			"filters",
			mcp.Required(),
			mcp.Description("Array of filter objects."),
			mcp.MinItems(1),
			mcp.Items(map[string]any{
				"type": "object",
			}),
		),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		filters, err := parseFilters(req.Params.Arguments["filters"])
		if err != nil {
			return invalidArguments(err)
		}

		items, err := client.FilterItems(filters)
		if err != nil {
			return toolFailure("filter items", err)
		}
		return mcp.NewToolResultText(formatItems(items)), nil
	})

	s.AddTool(mcp.NewTool("get_items_by_type",
		mcp.WithDescription("Get all Zealot items assigned to a specific type."),
		mcp.WithString("type_name", mcp.Required(), mcp.Description("The item type name, such as Ticket, Repeat, or Book.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		typeName, err := requiredNonEmptyString(req, "type_name")
		if err != nil {
			return invalidArguments(err)
		}

		items, err := client.GetItemsByType(typeName)
		if err != nil {
			return toolFailure("get items by type", err)
		}
		return mcp.NewToolResultText(formatItems(items)), nil
	})

	s.AddTool(mcp.NewTool("create_item",
		mcp.WithDescription("Create a new item. This MCP helper can also set initial attributes, content, and assign item types after creation."),
		mcp.WithString("title", mcp.Required(), mcp.Description("The new item title.")),
		mcp.WithString("content", mcp.Description("Optional initial content. This can be an empty string to leave content blank.")),
		mcp.WithObject("attributes", mcp.Description("Optional attribute map to set during creation.")),
		mcp.WithArray(
			"types",
			mcp.Description("Optional array of item type names to assign after creation."),
			mcp.Items(map[string]any{
				"type": "string",
			}),
		),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		title, err := requiredNonEmptyString(req, "title")
		if err != nil {
			return invalidArguments(err)
		}

		content, _, err := optionalArg[string](req, "content")
		if err != nil {
			return invalidArguments(err)
		}
		attributes, _, err := optionalArg[map[string]any](req, "attributes")
		if err != nil {
			return invalidArguments(err)
		}
		typeNames, _, err := optionalArg[[]string](req, "types")
		if err != nil {
			return invalidArguments(err)
		}

		item, err := client.CreateItem(CreateItemInput{
			Title:      title,
			Content:    content,
			Attributes: attributes,
			TypeNames:  typeNames,
		})
		if err != nil {
			return toolFailure("create item", err)
		}
		return mcp.NewToolResultText(formatItem(item)), nil
	})

	s.AddTool(mcp.NewTool("update_item",
		mcp.WithDescription("Update an existing item's title and/or content."),
		mcp.WithNumber("item_id", mcp.Required(), mcp.Description("The item ID to update.")),
		mcp.WithString("title", mcp.Description("Optional replacement title.")),
		mcp.WithString("content", mcp.Description("Optional replacement content. Provide an empty string to clear it.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		itemID, err := requiredArg[int](req, "item_id")
		if err != nil {
			return invalidArguments(err)
		}
		updates, err := collectArgMap(req, "title", "content")
		if err != nil {
			return invalidArguments(err)
		}

		if err := client.UpdateItemFields(itemID, updates); err != nil {
			return toolFailure("update item", err)
		}

		item, err := client.GetItemByID(itemID)
		if err != nil {
			return toolFailure("refresh updated item", err)
		}
		return mcp.NewToolResultText(formatItem(item)), nil
	})

	s.AddTool(mcp.NewTool("delete_item",
		mcp.WithDescription("Delete an item by ID."),
		mcp.WithNumber("item_id", mcp.Required(), mcp.Description("The item ID to delete.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		itemID, err := requiredArg[int](req, "item_id")
		if err != nil {
			return invalidArguments(err)
		}
		if err := client.DeleteItem(itemID); err != nil {
			return toolFailure("delete item", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Deleted item %d.", itemID)), nil
	})

	s.AddTool(mcp.NewTool("set_item_attributes",
		mcp.WithDescription("Set up to 10 attributes on an item in one call."),
		mcp.WithNumber("item_id", mcp.Required(), mcp.Description("The item ID to update.")),
		mcp.WithObject("attributes", mcp.Required(), mcp.Description("Object map of attribute keys to values.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		itemID, err := requiredArg[int](req, "item_id")
		if err != nil {
			return invalidArguments(err)
		}
		attributes, err := requiredArg[map[string]any](req, "attributes")
		if err != nil {
			return invalidArguments(err)
		}

		if err := client.SetItemAttributes(itemID, attributes); err != nil {
			return toolFailure("set item attributes", err)
		}

		item, err := client.GetItemByID(itemID)
		if err != nil {
			return toolFailure("refresh item after setting attributes", err)
		}
		return mcp.NewToolResultText(formatItem(item)), nil
	})

	s.AddTool(mcp.NewTool("rename_item_attribute",
		mcp.WithDescription("Rename an attribute key on an item."),
		mcp.WithNumber("item_id", mcp.Required(), mcp.Description("The item ID to update.")),
		mcp.WithString("old_key", mcp.Required(), mcp.Description("The current attribute key.")),
		mcp.WithString("new_key", mcp.Required(), mcp.Description("The new attribute key.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		itemID, err := requiredArg[int](req, "item_id")
		if err != nil {
			return invalidArguments(err)
		}
		oldKey, err := requiredNonEmptyString(req, "old_key")
		if err != nil {
			return invalidArguments(err)
		}
		newKey, err := requiredNonEmptyString(req, "new_key")
		if err != nil {
			return invalidArguments(err)
		}

		if err := client.RenameItemAttribute(itemID, oldKey, newKey); err != nil {
			return toolFailure("rename item attribute", err)
		}

		item, err := client.GetItemByID(itemID)
		if err != nil {
			return toolFailure("refresh item after renaming attribute", err)
		}
		return mcp.NewToolResultText(formatItem(item)), nil
	})

	s.AddTool(mcp.NewTool("delete_item_attribute",
		mcp.WithDescription("Delete a single attribute from an item."),
		mcp.WithNumber("item_id", mcp.Required(), mcp.Description("The item ID.")),
		mcp.WithString("key", mcp.Required(), mcp.Description("The attribute key to delete.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		itemID, err := requiredArg[int](req, "item_id")
		if err != nil {
			return invalidArguments(err)
		}
		key, err := requiredNonEmptyString(req, "key")
		if err != nil {
			return invalidArguments(err)
		}

		if err := client.DeleteItemAttribute(itemID, key); err != nil {
			return toolFailure("delete item attribute", err)
		}

		item, err := client.GetItemByID(itemID)
		if err != nil {
			return toolFailure("refresh item after deleting attribute", err)
		}
		return mcp.NewToolResultText(formatItem(item)), nil
	})

	s.AddTool(mcp.NewTool("assign_item_type",
		mcp.WithDescription("Assign an item type to an item."),
		mcp.WithNumber("item_id", mcp.Required(), mcp.Description("The item ID.")),
		mcp.WithString("type_name", mcp.Required(), mcp.Description("The item type name to assign.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		itemID, err := requiredArg[int](req, "item_id")
		if err != nil {
			return invalidArguments(err)
		}
		typeName, err := requiredNonEmptyString(req, "type_name")
		if err != nil {
			return invalidArguments(err)
		}

		if err := client.AssignItemType(itemID, typeName); err != nil {
			return toolFailure("assign item type", err)
		}

		item, err := client.GetItemByID(itemID)
		if err != nil {
			return toolFailure("refresh item after assigning type", err)
		}
		return mcp.NewToolResultText(formatItem(item)), nil
	})

	s.AddTool(mcp.NewTool("unassign_item_type",
		mcp.WithDescription("Remove an item type assignment from an item."),
		mcp.WithNumber("item_id", mcp.Required(), mcp.Description("The item ID.")),
		mcp.WithString("type_name", mcp.Required(), mcp.Description("The item type name to remove.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		itemID, err := requiredArg[int](req, "item_id")
		if err != nil {
			return invalidArguments(err)
		}
		typeName, err := requiredNonEmptyString(req, "type_name")
		if err != nil {
			return invalidArguments(err)
		}

		if err := client.UnassignItemType(itemID, typeName); err != nil {
			return toolFailure("unassign item type", err)
		}

		item, err := client.GetItemByID(itemID)
		if err != nil {
			return toolFailure("refresh item after unassigning type", err)
		}
		return mcp.NewToolResultText(formatItem(item)), nil
	})
}

func registerMetadataTools(s *server.MCPServer, client *ZealotClient) {
	s.AddTool(mcp.NewTool("list_item_types",
		mcp.WithDescription("List all item types available to the current account, including system types."),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		types, err := client.ListItemTypes()
		if err != nil {
			return toolFailure("list item types", err)
		}
		return mcp.NewToolResultText(formatJSON(types)), nil
	})

	s.AddTool(mcp.NewTool("create_item_type",
		mcp.WithDescription("Create an item type and optionally attach required attribute kinds."),
		mcp.WithString("name", mcp.Required(), mcp.Description("The new item type name.")),
		mcp.WithString("description", mcp.Description("Optional description.")),
		mcp.WithArray(
			"required_attribute_keys",
			mcp.Description("Optional array of attribute kind keys to require for this type."),
			mcp.Items(map[string]any{
				"type": "string",
			}),
		),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		name, err := requiredNonEmptyString(req, "name")
		if err != nil {
			return invalidArguments(err)
		}
		description, _, err := optionalArg[string](req, "description")
		if err != nil {
			return invalidArguments(err)
		}
		requiredKeys, _, err := optionalArg[[]string](req, "required_attribute_keys")
		if err != nil {
			return invalidArguments(err)
		}

		if err := client.CreateItemType(CreateItemTypeInput{
			Name:                  name,
			Description:           description,
			RequiredAttributeKeys: requiredKeys,
		}); err != nil {
			return toolFailure("create item type", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Created item type %q.", name)), nil
	})

	s.AddTool(mcp.NewTool("update_item_type",
		mcp.WithDescription("Update an item type's name and/or description."),
		mcp.WithNumber("type_id", mcp.Required(), mcp.Description("The item type ID.")),
		mcp.WithString("name", mcp.Description("Optional replacement name.")),
		mcp.WithString("description", mcp.Description("Optional replacement description. Provide an empty string to clear it.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		typeID, err := requiredArg[int](req, "type_id")
		if err != nil {
			return invalidArguments(err)
		}
		updates, err := collectArgMap(req, "name", "description")
		if err != nil {
			return invalidArguments(err)
		}

		if err := client.UpdateItemType(typeID, updates); err != nil {
			return toolFailure("update item type", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Updated item type %d.", typeID)), nil
	})

	s.AddTool(mcp.NewTool("delete_item_type",
		mcp.WithDescription("Delete an item type by ID."),
		mcp.WithNumber("type_id", mcp.Required(), mcp.Description("The item type ID.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		typeID, err := requiredArg[int](req, "type_id")
		if err != nil {
			return invalidArguments(err)
		}
		if err := client.DeleteItemType(typeID); err != nil {
			return toolFailure("delete item type", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Deleted item type %d.", typeID)), nil
	})

	s.AddTool(mcp.NewTool("add_item_type_attribute_kinds",
		mcp.WithDescription("Attach attribute kinds as required fields for an item type."),
		mcp.WithString("type_name", mcp.Required(), mcp.Description("The item type name.")),
		mcp.WithArray(
			"attribute_keys",
			mcp.Required(),
			mcp.Description("Array of attribute kind keys to attach."),
			mcp.MinItems(1),
			mcp.Items(map[string]any{
				"type": "string",
			}),
		),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		typeName, err := requiredNonEmptyString(req, "type_name")
		if err != nil {
			return invalidArguments(err)
		}
		attributeKeys, err := requiredArg[[]string](req, "attribute_keys")
		if err != nil {
			return invalidArguments(err)
		}
		if len(attributeKeys) == 0 {
			return invalidArguments(fmt.Errorf("attribute_keys must not be empty"))
		}

		if err := client.AddAttributeKindsToItemType(typeName, attributeKeys); err != nil {
			return toolFailure("attach attribute kinds to item type", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Attached %d attribute kind(s) to item type %q.", len(attributeKeys), typeName)), nil
	})

	s.AddTool(mcp.NewTool("remove_item_type_attribute_kinds",
		mcp.WithDescription("Remove required attribute kinds from an item type."),
		mcp.WithString("type_name", mcp.Required(), mcp.Description("The item type name.")),
		mcp.WithArray(
			"attribute_keys",
			mcp.Required(),
			mcp.Description("Array of attribute kind keys to remove."),
			mcp.MinItems(1),
			mcp.Items(map[string]any{
				"type": "string",
			}),
		),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		typeName, err := requiredNonEmptyString(req, "type_name")
		if err != nil {
			return invalidArguments(err)
		}
		attributeKeys, err := requiredArg[[]string](req, "attribute_keys")
		if err != nil {
			return invalidArguments(err)
		}
		if len(attributeKeys) == 0 {
			return invalidArguments(fmt.Errorf("attribute_keys must not be empty"))
		}

		if err := client.RemoveAttributeKindsFromItemType(typeName, attributeKeys); err != nil {
			return toolFailure("remove attribute kinds from item type", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Removed %d attribute kind(s) from item type %q.", len(attributeKeys), typeName)), nil
	})

	s.AddTool(mcp.NewTool("list_attribute_kinds",
		mcp.WithDescription("List all attribute kinds available to the current account, including system kinds."),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		kinds, err := client.ListAttributeKinds()
		if err != nil {
			return toolFailure("list attribute kinds", err)
		}
		return mcp.NewToolResultText(formatJSON(kinds)), nil
	})

	s.AddTool(mcp.NewTool("create_attribute_kind",
		mcp.WithDescription("Create a new attribute kind. For dropdown and list types, include a valid config object."),
		mcp.WithString("key", mcp.Required(), mcp.Description("The attribute key.")),
		mcp.WithString("base_type", mcp.Required(), mcp.Description("The base type, such as text, integer, decimal, dropdown, or list.")),
		mcp.WithString("description", mcp.Description("Optional description.")),
		mcp.WithObject("config", mcp.Description("Optional config object. Required by some base types, such as dropdown and list.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		key, err := requiredNonEmptyString(req, "key")
		if err != nil {
			return invalidArguments(err)
		}
		baseType, err := requiredNonEmptyString(req, "base_type")
		if err != nil {
			return invalidArguments(err)
		}
		description, _, err := optionalArg[string](req, "description")
		if err != nil {
			return invalidArguments(err)
		}
		config, _, err := optionalArg[json.RawMessage](req, "config")
		if err != nil {
			return invalidArguments(err)
		}

		if err := client.CreateAttributeKind(AttributeKind{
			Key:         key,
			BaseType:    baseType,
			Description: description,
			Config:      config,
		}); err != nil {
			return toolFailure("create attribute kind", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Created attribute kind %q.", key)), nil
	})

	s.AddTool(mcp.NewTool("update_attribute_kind",
		mcp.WithDescription("Update an attribute kind's key, description, and/or base type."),
		mcp.WithNumber("kind_id", mcp.Required(), mcp.Description("The attribute kind ID.")),
		mcp.WithString("key", mcp.Description("Optional replacement key.")),
		mcp.WithString("description", mcp.Description("Optional replacement description. Provide an empty string to clear it.")),
		mcp.WithString("base_type", mcp.Description("Optional replacement base type.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		kindID, err := requiredArg[int](req, "kind_id")
		if err != nil {
			return invalidArguments(err)
		}
		updates, err := collectArgMap(req, "key", "description", "base_type")
		if err != nil {
			return invalidArguments(err)
		}

		if err := client.UpdateAttributeKind(kindID, updates); err != nil {
			return toolFailure("update attribute kind", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Updated attribute kind %d.", kindID)), nil
	})

	s.AddTool(mcp.NewTool("update_attribute_kind_config",
		mcp.WithDescription("Replace the config JSON for an attribute kind."),
		mcp.WithNumber("kind_id", mcp.Required(), mcp.Description("The attribute kind ID.")),
		mcp.WithObject("config", mcp.Required(), mcp.Description("The new config object.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		kindID, err := requiredArg[int](req, "kind_id")
		if err != nil {
			return invalidArguments(err)
		}
		config, err := requiredArg[json.RawMessage](req, "config")
		if err != nil {
			return invalidArguments(err)
		}

		if err := client.UpdateAttributeKindConfig(kindID, config); err != nil {
			return toolFailure("update attribute kind config", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Updated config for attribute kind %d.", kindID)), nil
	})

	s.AddTool(mcp.NewTool("delete_attribute_kind",
		mcp.WithDescription("Delete a non-system attribute kind by ID."),
		mcp.WithNumber("kind_id", mcp.Required(), mcp.Description("The attribute kind ID.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		kindID, err := requiredArg[int](req, "kind_id")
		if err != nil {
			return invalidArguments(err)
		}
		if err := client.DeleteAttributeKind(kindID); err != nil {
			return toolFailure("delete attribute kind", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Deleted attribute kind %d.", kindID)), nil
	})
}

func registerPlannerTools(s *server.MCPServer, client *ZealotClient) {
	s.AddTool(mcp.NewTool("get_planner_day",
		mcp.WithDescription("Get planner items for a specific calendar day."),
		mcp.WithString("date", mcp.Required(), mcp.Description("Date in YYYY-MM-DD format.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		date, err := requiredNonEmptyString(req, "date")
		if err != nil {
			return invalidArguments(err)
		}

		items, err := client.GetPlannerDay(date)
		if err != nil {
			return toolFailure("get planner day", err)
		}
		return mcp.NewToolResultText(formatItems(items)), nil
	})

	s.AddTool(mcp.NewTool("get_planner_week",
		mcp.WithDescription("Get planner items for a week. The week string should be whatever Zealot's week parser accepts, typically a week code or a date in that week."),
		mcp.WithString("week", mcp.Required(), mcp.Description("Week identifier accepted by Zealot.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		week, err := requiredNonEmptyString(req, "week")
		if err != nil {
			return invalidArguments(err)
		}

		items, err := client.GetPlannerWeek(week)
		if err != nil {
			return toolFailure("get planner week", err)
		}
		return mcp.NewToolResultText(formatItems(items)), nil
	})

	s.AddTool(mcp.NewTool("get_planner_month",
		mcp.WithDescription("Get planner items for a month and year."),
		mcp.WithNumber("month", mcp.Required(), mcp.Description("Month number, 1 through 12.")),
		mcp.WithNumber("year", mcp.Required(), mcp.Description("Four-digit year.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		month, err := requiredArg[int](req, "month")
		if err != nil {
			return invalidArguments(err)
		}
		year, err := requiredArg[int](req, "year")
		if err != nil {
			return invalidArguments(err)
		}

		items, err := client.GetPlannerMonth(month, year)
		if err != nil {
			return toolFailure("get planner month", err)
		}
		return mcp.NewToolResultText(formatItems(items)), nil
	})

	s.AddTool(mcp.NewTool("get_planner_year",
		mcp.WithDescription("Get planner items for a full year."),
		mcp.WithNumber("year", mcp.Required(), mcp.Description("Four-digit year.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		year, err := requiredArg[int](req, "year")
		if err != nil {
			return invalidArguments(err)
		}

		items, err := client.GetPlannerYear(year)
		if err != nil {
			return toolFailure("get planner year", err)
		}
		return mcp.NewToolResultText(formatItems(items)), nil
	})
}

func registerRepeatTools(s *server.MCPServer, client *ZealotClient) {
	s.AddTool(mcp.NewTool("get_repeats_for_day",
		mcp.WithDescription("Get repeat items scheduled for a given day, including completion state."),
		mcp.WithString("date", mcp.Required(), mcp.Description("Date in YYYY-MM-DD format.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		date, err := requiredNonEmptyString(req, "date")
		if err != nil {
			return invalidArguments(err)
		}

		repeats, err := client.GetRepeatsForDay(date)
		if err != nil {
			return toolFailure("get repeats for day", err)
		}
		return mcp.NewToolResultText(formatJSON(repeats)), nil
	})

	s.AddTool(mcp.NewTool("set_repeat_status",
		mcp.WithDescription("Set the completion status for a repeat item on a specific day. Use status \"Not Completed\" to clear the entry."),
		mcp.WithNumber("item_id", mcp.Required(), mcp.Description("The repeat item ID.")),
		mcp.WithString("date", mcp.Required(), mcp.Description("Date in YYYY-MM-DD format.")),
		mcp.WithString("status", mcp.Required(), mcp.Description("Status value such as Complete, Partial, Skipped, or Not Completed.")),
		mcp.WithString("comment", mcp.Description("Optional comment to store with the repeat status.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		itemID, err := requiredArg[int](req, "item_id")
		if err != nil {
			return invalidArguments(err)
		}
		date, err := requiredNonEmptyString(req, "date")
		if err != nil {
			return invalidArguments(err)
		}
		status, err := requiredNonEmptyString(req, "status")
		if err != nil {
			return invalidArguments(err)
		}
		comment, _, err := optionalArg[string](req, "comment")
		if err != nil {
			return invalidArguments(err)
		}

		if err := client.SetRepeatStatus(itemID, date, status, comment); err != nil {
			return toolFailure("set repeat status", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Set repeat status for item %d on %s to %q.", itemID, date, status)), nil
	})
}

func registerCommentTools(s *server.MCPServer, client *ZealotClient) {
	s.AddTool(mcp.NewTool("get_comments_for_day",
		mcp.WithDescription("Get all comment entries recorded on a specific day."),
		mcp.WithString("date", mcp.Required(), mcp.Description("Date in YYYY-MM-DD format.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		date, err := requiredNonEmptyString(req, "date")
		if err != nil {
			return invalidArguments(err)
		}

		entries, err := client.GetCommentsForDay(date)
		if err != nil {
			return toolFailure("get comments for day", err)
		}
		return mcp.NewToolResultText(formatJSON(entries)), nil
	})

	s.AddTool(mcp.NewTool("get_comments_for_item",
		mcp.WithDescription("Get all comment entries attached to an item."),
		mcp.WithNumber("item_id", mcp.Required(), mcp.Description("The item ID.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		itemID, err := requiredArg[int](req, "item_id")
		if err != nil {
			return invalidArguments(err)
		}

		entries, err := client.GetCommentsForItem(itemID)
		if err != nil {
			return toolFailure("get comments for item", err)
		}
		return mcp.NewToolResultText(formatJSON(entries)), nil
	})

	s.AddTool(mcp.NewTool("add_comment",
		mcp.WithDescription("Add a comment entry to an item."),
		mcp.WithNumber("item_id", mcp.Required(), mcp.Description("The item ID.")),
		mcp.WithString("content", mcp.Required(), mcp.Description("Comment text.")),
		mcp.WithString("timestamp", mcp.Description("Optional RFC3339 timestamp. If omitted, the server uses now.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		itemID, err := requiredArg[int](req, "item_id")
		if err != nil {
			return invalidArguments(err)
		}
		content, err := requiredArg[string](req, "content")
		if err != nil {
			return invalidArguments(err)
		}
		timestamp, _, err := optionalArg[string](req, "timestamp")
		if err != nil {
			return invalidArguments(err)
		}

		if err := client.AddComment(itemID, timestamp, content); err != nil {
			return toolFailure("add comment", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Added comment to item %d.", itemID)), nil
	})

	s.AddTool(mcp.NewTool("update_comment",
		mcp.WithDescription("Update a comment entry's content, optionally replacing its timestamp."),
		mcp.WithNumber("comment_id", mcp.Required(), mcp.Description("The comment ID.")),
		mcp.WithString("content", mcp.Required(), mcp.Description("Replacement comment text. This may be an empty string.")),
		mcp.WithString("timestamp", mcp.Description("Optional RFC3339 timestamp.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		commentID, err := requiredArg[int64](req, "comment_id")
		if err != nil {
			return invalidArguments(err)
		}
		content, err := requiredArg[string](req, "content")
		if err != nil {
			return invalidArguments(err)
		}
		timestamp, _, err := optionalArg[string](req, "timestamp")
		if err != nil {
			return invalidArguments(err)
		}

		if err := client.UpdateComment(commentID, content, timestamp); err != nil {
			return toolFailure("update comment", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Updated comment %d.", commentID)), nil
	})

	s.AddTool(mcp.NewTool("delete_comment",
		mcp.WithDescription("Delete a comment entry."),
		mcp.WithNumber("comment_id", mcp.Required(), mcp.Description("The comment ID.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		commentID, err := requiredArg[int64](req, "comment_id")
		if err != nil {
			return invalidArguments(err)
		}
		if err := client.DeleteComment(commentID); err != nil {
			return toolFailure("delete comment", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Deleted comment %d.", commentID)), nil
	})
}

func registerTrackerTools(s *server.MCPServer, client *ZealotClient) {
	s.AddTool(mcp.NewTool("get_tracker_day",
		mcp.WithDescription("Get tracker entries for a specific day."),
		mcp.WithString("date", mcp.Required(), mcp.Description("Date in YYYY-MM-DD format.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		date, err := requiredNonEmptyString(req, "date")
		if err != nil {
			return invalidArguments(err)
		}

		entries, err := client.GetTrackerDay(date)
		if err != nil {
			return toolFailure("get tracker entries for day", err)
		}
		return mcp.NewToolResultText(formatJSON(entries)), nil
	})

	s.AddTool(mcp.NewTool("add_tracker_entry",
		mcp.WithDescription("Add a tracker entry for a Tracker-type item."),
		mcp.WithNumber("item_id", mcp.Required(), mcp.Description("The Tracker item ID.")),
		mcp.WithString("timestamp", mcp.Description("Optional RFC3339 timestamp. If omitted, the server uses now.")),
		mcp.WithNumber("level", mcp.Description("Optional level from 1 to 10. If omitted, the server defaults to 3.")),
		mcp.WithString("comment", mcp.Description("Optional comment.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		itemID, err := requiredArg[int](req, "item_id")
		if err != nil {
			return invalidArguments(err)
		}
		timestamp, _, err := optionalArg[string](req, "timestamp")
		if err != nil {
			return invalidArguments(err)
		}
		level, _, err := optionalArg[int](req, "level")
		if err != nil {
			return invalidArguments(err)
		}
		comment, _, err := optionalArg[string](req, "comment")
		if err != nil {
			return invalidArguments(err)
		}

		if err := client.AddTrackerEntry(itemID, timestamp, level, comment); err != nil {
			return toolFailure("add tracker entry", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Added tracker entry for item %d.", itemID)), nil
	})

	s.AddTool(mcp.NewTool("delete_tracker_entry",
		mcp.WithDescription("Delete a tracker entry."),
		mcp.WithNumber("tracker_id", mcp.Required(), mcp.Description("The tracker entry ID.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		trackerID, err := requiredArg[int64](req, "tracker_id")
		if err != nil {
			return invalidArguments(err)
		}
		if err := client.DeleteTrackerEntry(trackerID); err != nil {
			return toolFailure("delete tracker entry", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Deleted tracker entry %d.", trackerID)), nil
	})

	s.AddTool(mcp.NewTool("get_score_history",
		mcp.WithDescription("Get the analysis score series for the last 30 days."),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		entries, err := client.GetScoreHistory()
		if err != nil {
			return toolFailure("get score history", err)
		}
		return mcp.NewToolResultText(formatJSON(entries)), nil
	})
}

func registerMediaTools(s *server.MCPServer, client *ZealotClient) {
	s.AddTool(mcp.NewTool("list_media",
		mcp.WithDescription("List files and folders in a media directory. Use an empty or omitted path to list the media root."),
		mcp.WithString("path", mcp.Description("Optional relative media directory path.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		path, _, err := optionalArg[string](req, "path")
		if err != nil {
			return invalidArguments(err)
		}

		listing, err := client.ListMedia(path)
		if err != nil {
			return toolFailure("list media", err)
		}
		return mcp.NewToolResultText(formatJSON(listing)), nil
	})

	s.AddTool(mcp.NewTool("create_media_folder",
		mcp.WithDescription("Create a folder in the account media area."),
		mcp.WithString("folder", mcp.Required(), mcp.Description("Relative folder path to create.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		folder, err := requiredNonEmptyString(req, "folder")
		if err != nil {
			return invalidArguments(err)
		}
		if err := client.CreateMediaFolder(folder); err != nil {
			return toolFailure("create media folder", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Created media folder %q.", folder)), nil
	})

	s.AddTool(mcp.NewTool("upload_media_file",
		mcp.WithDescription("Upload a local file from the machine running this MCP server into the Zealot media area."),
		mcp.WithString("local_path", mcp.Required(), mcp.Description("Absolute or relative local filesystem path to the file.")),
		mcp.WithString("destination_dir", mcp.Description("Optional relative media directory to upload into. Defaults to the media root.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		localPath, err := requiredNonEmptyString(req, "local_path")
		if err != nil {
			return invalidArguments(err)
		}
		destinationDir, _, err := optionalArg[string](req, "destination_dir")
		if err != nil {
			return invalidArguments(err)
		}

		if err := client.UploadMediaFile(localPath, destinationDir); err != nil {
			return toolFailure("upload media file", err)
		}
		if destinationDir == "" {
			return mcp.NewToolResultText(fmt.Sprintf("Uploaded %q to the media root.", localPath)), nil
		}
		return mcp.NewToolResultText(fmt.Sprintf("Uploaded %q to %q.", localPath, destinationDir)), nil
	})

	s.AddTool(mcp.NewTool("rename_media_entry",
		mcp.WithDescription("Rename a file or folder while keeping it in the same media directory."),
		mcp.WithString("old_location", mcp.Required(), mcp.Description("Current relative path.")),
		mcp.WithString("new_name", mcp.Required(), mcp.Description("New file or folder name, not a full path.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		oldLocation, err := requiredNonEmptyString(req, "old_location")
		if err != nil {
			return invalidArguments(err)
		}
		newName, err := requiredNonEmptyString(req, "new_name")
		if err != nil {
			return invalidArguments(err)
		}

		if err := client.RenameMediaEntry(oldLocation, newName); err != nil {
			return toolFailure("rename media entry", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Renamed media entry %q to %q.", oldLocation, newName)), nil
	})

	s.AddTool(mcp.NewTool("delete_media_entry",
		mcp.WithDescription("Delete a media file or folder by relative path."),
		mcp.WithString("path", mcp.Required(), mcp.Description("Relative media path to delete.")),
	), func(ctx context.Context, req mcp.CallToolRequest) (*mcp.CallToolResult, error) {
		path, err := requiredNonEmptyString(req, "path")
		if err != nil {
			return invalidArguments(err)
		}
		if err := client.DeleteMediaEntry(path); err != nil {
			return toolFailure("delete media entry", err)
		}
		return mcp.NewToolResultText(fmt.Sprintf("Deleted media entry %q.", path)), nil
	})
}
