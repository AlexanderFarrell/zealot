import { BasicAPI, patch_json, delete_req } from "@websoil/engine";
import { LazyData } from "@websoil/engine/src/api/api_helper";
import { AttributeKind } from "@zealot/domain/src/attribute";
import type {
    AttributeKindDto,
    AddAttributeKindDto,
    UpdateAttributeKindDto,
} from "@zealot/domain/src/attribute";

export class AttributeKindAPI extends BasicAPI<AttributeKind, AttributeKindDto, AddAttributeKindDto, UpdateAttributeKindDto> {
    public Kinds: LazyData<Record<string, AttributeKind>>;

    constructor(baseURL: string) {
        let dto_factory = (dto: AttributeKindDto) => {
            return new AttributeKind(dto);
        }

        let kinds_source = async () => {
            let kinds: Record<string, AttributeKind> = {};
            let list = await this.get_all();
            list.forEach(kind => {
                kinds[kind.Key] = kind;
            })
            return kinds;
        }

        super(`${baseURL}/attribute`, dto_factory)
        this.Kinds = new LazyData(kinds_source)
    }

    // Server route is PATCH /attribute/id/{kind_id}
    override async update(id: number, updates: UpdateAttributeKindDto) {
        return patch_json(`${this.URL}/id/${id}`, updates);
    }

    // Server route is DELETE /attribute/key/{key}
    async removeByKey(key: string) {
        return delete_req(`${this.URL}/key/${key}`);
    }
}
