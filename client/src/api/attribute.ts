import { delete_req, patch_req } from "../core/api_helper";

const AttributeAPI = {
    rename: async (item_id: number, old_key: string, new_key: string) => {
        await patch_req(`/api/item/${item_id}/attr/rename`, {
            old_key: old_key,
            new_key: new_key
        })
    },

    set_value: async (item_id: number, key: string, value: any) => {
        let data: any = {}
        data[key] = value;
        await patch_req(`/api/item/${item_id}/attr`, data)
    },

    remove: async (item_id: number, key: string) => {
        await delete_req(`/api/item/${item_id}/attr/${key}`)
    }
}

export default AttributeAPI;
