import { BasicAPI } from "@websoil/engine";
import { delete_req, get_json, LazyData, patch_json, post_json, post_req } from "@websoil/engine/src/api/api_helper";
import { ItemType, ItemTypeSummary } from "@zealot/domain/src/item_type";
import type {
    AddItemTypeDto,
    ItemTypeDto,
    ItemTypeSummaryDto,
    UpdateItemTypeDto,
} from "@zealot/domain/src/item_type";


export class ItemTypeAPI extends BasicAPI<ItemType, ItemTypeDto,
    AddItemTypeDto, UpdateItemTypeDto>
{
    private static Instances = new Set<ItemTypeAPI>();
    public Types: LazyData<Record<string, ItemType>>;
    public Summaries: LazyData<ItemTypeSummary[]>;

    constructor(baseURL: string) {
        let dto_factory = (dto: ItemTypeDto) => {
            return new ItemType(dto);
        }
        let summary_factory = (dto: ItemTypeSummaryDto) => {
            return new ItemTypeSummary(dto);
        }

        let types_source = async () => {
            let types: Record<string, ItemType> = {};
            let list = await this.get_all();
            list.forEach(t => {
                types[t.Name] = t;
            })
            return types;
        }
        let summaries_source = async () => {
            let list = await get_json(`${this.URL}/summary`) as ItemTypeSummaryDto[];
            return list.map(summary_factory);
        }

        super(`${baseURL}/item_type`, dto_factory)
        this.Types = new LazyData(types_source)
        this.Summaries = new LazyData(summaries_source)
        ItemTypeAPI.Instances.add(this);
    }

    private static invalidateAll() {
        for (const instance of ItemTypeAPI.Instances) {
            instance.Types.MarkDirty();
            instance.Summaries.MarkDirty();
        }
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

    async get_by_name(name: string): Promise<ItemType | null> {
        try {
            const dto = await get_json(`${this.URL}/name/${encodeURIComponent(name)}`) as ItemTypeDto;
            return this.Factory(dto);
        } catch {
            return null;
        }
    }

    async get_summaries(): Promise<ItemTypeSummary[]> {
        return this.Summaries.Get();
    }

    async create(dto: AddItemTypeDto): Promise<ItemType> {
        const data = await post_json(this.URL, dto) as ItemTypeDto;
        ItemTypeAPI.invalidateAll();
        return this.Factory(data);
    }

    async update(id: number, updates: UpdateItemTypeDto): Promise<ItemType> {
        const data = await patch_json(`${this.URL}/${id}`, updates) as ItemTypeDto;
        ItemTypeAPI.invalidateAll();
        return this.Factory(data);
    }

    async remove(id: number): Promise<Response> {
        const response = await delete_req(`${this.URL}/${id}`);
        ItemTypeAPI.invalidateAll();
        return response;
    }

    async assign(type_id: number, attribute_keys: string[]) {
        const response = await post_req(`${this.URL}/${type_id}/attr_kind`, attribute_keys);
        ItemTypeAPI.invalidateAll();
        return response;
    }

    async unassign(type_id: number, attribute_keys: string[]) {
        const response = await delete_req(`${this.URL}/${type_id}/attr_kind`, attribute_keys);
        ItemTypeAPI.invalidateAll();
        return response;
    }
}
