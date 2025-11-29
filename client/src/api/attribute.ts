

const AttributeAPI = {
    rename: async (item_id: number, old_key: string, new_key: string) => {
        await fetch(`/api/item/${item_id}/attr/rename`, {
            method: "PATCH",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                old_key: old_key,
                new_key: new_key
            })
        })
    },

    set_value: async (item_id: number, key: string, value: any) => {
        let data: any = {}
        data[key] = value;
        await fetch(`/api/item/${item_id}/attr`, {
            method: "PATCH",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify(data)
        })
    },

    remove: async (item_id: number, key: string) => {
        await fetch(`/api/item/${item_id}/attr/${key}`, {
            method: "DELETE"
        })
    }
}

export default AttributeAPI;