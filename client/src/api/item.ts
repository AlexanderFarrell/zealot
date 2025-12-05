import AttributeAPI from "./attribute";
import AttributeKindAPI from "./attribute_kind";
import { ItemTypeAPI } from "./item_type";

export const ItemAPI = {
    get: async (id: number) => {
        return (await fetch(`/api/item/id/${id}`)).json();
    },

    get_by_title: async (title: string) => {
        return (await fetch(`/api/item/title/${title}`)).json();
    },

    search: async (term: string) => {
        return (await fetch(`/api/item/search?term=${term}`)).json();
    },

    add: async (title: string) => {
        let response = await fetch(`/api/item`, {
            method: "POST",
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({'title': title})
        });
        return response.ok;
    },

    update: async (item_id: number, fields: object) => {
        let response = await fetch(`/api/item/${item_id}`, {
            method: "PATCH",
            headers: {
                'Content-Type': "application/json"
            },
            body: JSON.stringify(fields)
        })
        return response.ok;
    },

    remove: async (id: number) => {
        let response = await fetch(`/api/item/${id}`, {
            method: "DELETE",
            headers: {
                "Content-Type": "application/json"
            },
        })
        return response.ok;
    },

    Attributes: AttributeAPI,
    AttributeKinds: AttributeKindAPI,
    Types: ItemTypeAPI
}

export default ItemAPI;