import { delete_req, get_json, patch_json, post_req } from "../core/api_helper";

const AttributeKindAPI = {
    get_all: async () => {
        return get_json(`/api/item/kind`)
    },

    add: async (key: string, 
        description: string, 
        base_type: string) => {
        return post_req(`/api/item/kind`,
            {
                key,
                description,
                base_type,
                config: {}
            }
        )
    },

    update: async (kind_id: number, updates: any) => {
        return patch_json(`/api/item/kind/${kind_id}`, updates)
    },

    remove: async (kind_id: number) => {
        return delete_req(`/api/item/kind/${kind_id}`)
    }
}

export default AttributeKindAPI;