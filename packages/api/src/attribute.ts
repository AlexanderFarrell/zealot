import { BaseAPI } from "./common";
import { delete_req, patch_req } from "@websoil/engine";

export class AttributeAPI extends BaseAPI {
    public constructor(baseURL: string) {
        super(baseURL)
    }

    public async rename(item_id: number, old_key: string, new_key: string) {
        await patch_req(`${this.baseUrl}/item/${item_id}/attr/rename`, {
            old_key,
            new_key
        })
    }

    public async set_value(item_id: number, key: string, value: string) {
        await patch_req(`${this.baseUrl}/item/${item_id}/attr`, {
            [key]: value
        })
    }

    public async remove(item_id: number, key: string) {
        await delete_req(`${this.baseUrl}/item/${item_id}/attr/${key}`)
    }
}