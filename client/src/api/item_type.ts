import { BasicAPI, delete_req, post_req } from "../core/api_helper";

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

export const ItemTypeAPI: ItemTypeAPIHandler = new ItemTypeAPIHandler();