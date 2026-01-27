import { delete_req, get_json, patch_req, post_json, post_req } from "../core/api_helper";
import AttributeAPI from "./attribute";
import AttributeKindAPI from "./attribute_kind";
import { ItemTypeAPI, type ItemType } from "./item_type";

export interface Item {
    item_id: number,
    title: string,
    content: string,
    attributes?: Record<string, any>,
    types?: Array<ItemType>,
    children?: Array<Item>
}

export type AttributeFilter = {
    key: string;
    op?: "eq" | "ne" | "gt" | "gte" | "lt" | "lte"
        | "ilike" | "=" | "<" | ">" | "<=" | ">=" 
        | "<>" | "!=";
    value: any;
    list_mode?: "any" | "all" | "none";
}

export const ItemAPI = {
    get: async (id: number): Promise<Item> => {
        return get_json(`/api/item/id/${id}`);
    },

    get_by_title: async (title: string): Promise<Item> => {
        return get_json(`/api/item/title/${title}`);
    },

    search: async (term: string): Promise<Item[]> => {
        return get_json(`/api/item/search?term=${term}`);
    },

    root_items: async (): Promise<Item[]> => {
        return get_json(`/api/item/`);
    },

    get_by_type: async (item_type: string): Promise<Item[]> => {
        return get_json(`/api/item?type=${item_type}`);
    },

    filter: async (filters: AttributeFilter[]): Promise<Item[]> => {
        return post_json(`/api/item/filter`, {filters});
    },

    related: async (parent_id: number): Promise<Item[]> => {
        return get_json(`/api/item/related/${parent_id}`)
    },

    children: async (parent_id: number): Promise<Item[]> => {
        return get_json(`/api/item/children/${parent_id}`)
    },

    add: async (item: Item): Promise<Item> => {
        return post_json(`/api/item`, item)
    },

    update: async (item_id: number, fields: object) => {
        let response = await patch_req(`/api/item/${item_id}`, fields);
        return response.ok;
    },

    remove: async (id: number) => {
        let response = await delete_req(`/api/item/${id}`);
        return response.ok;
    },

    assign_type: async (id: number, typeName: string) => {
        return post_req(`/api/item/${id}/assign_type/${typeName}`, {});
    },

    unassign_type: async (id: number, typeName: string) => {
        return delete_req(`/api/item/${id}/assign_type/${typeName}`)
    },

    Attributes: AttributeAPI,
    AttributeKinds: AttributeKindAPI,
    Types: ItemTypeAPI
}

export default ItemAPI;
