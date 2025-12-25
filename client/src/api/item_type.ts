import { BasicAPI, delete_req, post_req } from "../core/api_helper";
import { events } from "../core/events";

export interface ItemType {
    type_id?: number,
    name: string,
    description: string,
    is_system?: boolean,
    required_attribute_keys?: string[],
}

class ItemTypeAPIHandler extends BasicAPI<ItemType> {
    constructor() {
        super("/api/item/type")
    }

    assign(attribute_keys: string[], item_type_name: string) {
        return post_req(`${this.URL}/assign/${item_type_name}`, {
            attribute_kinds: attribute_keys
        })
    }

    unassign(attribute_keys: string[], item_type_name: string) {
        return delete_req(`${this.URL}/assign/${item_type_name}`, {
            attribute_kinds: attribute_keys
        })
    }
}
// Store attribute kinds for validation
let types: ItemType[] | null = null;
let types_in_flight: Promise<ItemType[]> | null = null;


events.on('refresh_attributes', () => {
    // Mark stale
    types = null;
    if (!types_in_flight) {
        types_in_flight = ItemTypeAPI.get_all().then(data => {
            types = data;
            types_in_flight = null;
            return types;
        }).catch(err => {
            types_in_flight = null;
            throw err;
        })
    }
})
export async function get_item_types(force_refresh: boolean = false) {
    if (!force_refresh && types) {
        return types;
    }

    if (types_in_flight) {
        return types_in_flight;
    }

    types_in_flight = ItemTypeAPI.get_all().then(data => {
        types = data;
        types_in_flight = null;
        return types;
    }).catch(err => {
        types_in_flight = null;
        throw err;
    })

    return types_in_flight;
}

export async function get_type_for_name(key: string) {
    const data = await get_item_types();
    return data.find(e => e.name === key);
}


export const ItemTypeAPI: ItemTypeAPIHandler = new ItemTypeAPIHandler();
export default ItemTypeAPI;