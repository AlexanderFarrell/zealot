import { BasicAPI } from "@websoil/engine";
import { delete_req, get_json, LazyData, post_req } from "@websoil/engine/src/api/api_helper";
import { ItemType } from "@zealot/domain/src/item_type";
import type { AddItemTypeDto, ItemTypeDto, UpdateItemTypeDto } from "@zealot/domain/src/item_type";


export class ItemTypeAPI extends BasicAPI<ItemType, ItemTypeDto,
    AddItemTypeDto, UpdateItemTypeDto>
{
    public Types: LazyData<Record<string, ItemType>>;

    constructor(baseURL: string) {
        let dto_factory = (dto: ItemTypeDto) => {
            return new ItemType(dto);
        }

        let types_source = async () => {
            let types: Record<string, ItemType> = {};
            let list = await this.get_all();
            list.forEach(t => {
                types[t.Name] = t;
            })
            return types;
        }

        super(`${baseURL}/item_type`, dto_factory)
        this.Types = new LazyData(types_source)
    }

    // Backend uses GET /item_type/{type_id} (no /id/ segment)
    async get_by_id(id: number): Promise<ItemType | null> {
        try {
            const dto = await get_json(`${this.URL}/${id}`) as ItemTypeDto;
            return this.Factory(dto);
        } catch {
            return null;
        }
    }

    async assign(type_id: number, attribute_keys: string[]) {
        return post_req(`${this.URL}/${type_id}/attr_kind`, attribute_keys)
    }

    async unassign(type_id: number, attribute_keys: string[]) {
        return delete_req(`${this.URL}/${type_id}/attr_kind`, attribute_keys)
    }
}
