import { BasicAPI } from "../core/api_helper";

export interface ItemType {
    type_id?: number,
    name: string,
    description: string,
    required_attribute_keys?: string[],
}

export const ItemTypeAPI: BasicAPI<ItemType> = new BasicAPI<ItemType>("/api/item/type");