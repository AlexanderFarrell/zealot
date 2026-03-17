import { BasicAPI, patch_json } from "@websoil/engine";
import { LazyData } from "@websoil/engine/src/api/api_helper";
import { AddAttributeKindDto, AttributeConfig, 
    AttributeKind, 
    AttributeKindDto, UpdateAttributeKindDto } from "@zealot/domain/src/attribute";

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

        super(`${baseURL}/item/kind`, dto_factory)
        this.Kinds = new LazyData(kinds_source)
    }

    async UpdateConfig(kind_id: number, config: AttributeConfig) {
        return patch_json(`${this.URL}/${kind_id}/config`, {config})
    }
}