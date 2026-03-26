import { BasicAPI } from "@websoil/engine";
import { delete_req, LazyData, post_req } from "@websoil/engine/src/api/api_helper";
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

        super(`${baseURL}/item/type`, dto_factory)
        this.Types = new LazyData(types_source)
    }

    async assign(attribute_keys: string[], item_type_name: string) {
        return post_req(`${this.URL}/assign/${item_type_name}`, {
            attribute_kinds: attribute_keys
        })
    }

    async unassign(attribute_keys: string[], item_type_name: string) {
        return delete_req(`${this.URL}/assign/${item_type_name}`, {
            attribute_kinds: attribute_keys
        })
    }
}
