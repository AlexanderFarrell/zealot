import { delete_req, get_json, patch_req, post_json, post_req } from "@websoil/engine";
import { Item } from "@zealot/domain/src/item";
import type { AddItemDto, ItemDto, UpdateItemDto } from "@zealot/domain/src/item";
import { BaseAPI } from "./common";

export type SearchScope = 'title' | 'content' | 'heading';

export interface SearchResultDto extends ItemDto {
    match_scope: SearchScope;
    snippet?: string;
}

export class SearchResult {
    public readonly item: Item;
    public readonly matchScope: SearchScope;
    public readonly snippet: string | null;

    constructor(dto: SearchResultDto) {
        this.item = new Item(dto);
        this.matchScope = dto.match_scope;
        this.snippet = dto.snippet ?? null;
    }
}

export interface MostViewedItemDto {
    item_id: number;
    title: string;
    view_count: number;
}

export interface MostViewedEntry {
    item: Item;
    viewCount: number;
}

export interface AttributeFilterDto {
    key: string;
    op: string;        // "eq" | "ne" | "gt" | "lt" | "gte" | "lte"
    value: any;
    list_mode: string; // "Any" | "All" | "None"
}

export class ItemAPI extends BaseAPI {
    public constructor(baseURL: string) {
        super(baseURL)
    }

    async GetAll(type?: string): Promise<Item[]> {
        const url = type
            ? `${this.baseUrl}/item?type=${encodeURIComponent(type)}`
            : `${this.baseUrl}/item`;
        const dtos: ItemDto[] = await get_json(url);
        return dtos.map(d => new Item(d));
    }

    async GetByTitle(title: string): Promise<Item> {
        const dto = await get_json(`${this.baseUrl}/item/title/${encodeURIComponent(title)}`) as ItemDto;
        return new Item(dto);
    }

    async GetById(item_id: number): Promise<Item> {
        const dto = await get_json(`${this.baseUrl}/item/id/${item_id}`) as ItemDto;
        return new Item(dto);
    }

    async GetRecent(limit = 30, offset = 0): Promise<Item[]> {
        const dtos: ItemDto[] = await get_json(`${this.baseUrl}/item/recent?limit=${limit}&offset=${offset}`);
        return dtos.map(d => new Item(d));
    }

    async Search(term: string, options?: { scope?: SearchScope; regex?: boolean; limit?: number; offset?: number }): Promise<SearchResult[]> {
        const scope = options?.scope ?? 'title';
        let url = `${this.baseUrl}/item/search?term=${encodeURIComponent(term)}&scope=${scope}`;
        if (options?.regex) url += `&regex=true`;
        if (options?.limit != null) url += `&limit=${options.limit}`;
        if (options?.offset != null) url += `&offset=${options.offset}`;
        const dtos: SearchResultDto[] = await get_json(url);
        return dtos.map(d => new SearchResult(d));
    }

    async GetChildren(item_id: number): Promise<Item[]> {
        const dtos: ItemDto[] = await get_json(`${this.baseUrl}/item/children/${item_id}`);
        return dtos.map(d => new Item(d));
    }

    async GetRelated(item_id: number): Promise<Item[]> {
        const dtos: ItemDto[] = await get_json(`${this.baseUrl}/item/related/${item_id}`);
        return dtos.map(d => new Item(d));
    }

    async GetBacklinks(item_id: number): Promise<Item[]> {
        const dtos: ItemDto[] = await get_json(`${this.baseUrl}/item/backlinks/${item_id}`);
        return dtos.map(d => new Item(d));
    }

    async Filter(filters: AttributeFilterDto[]): Promise<Item[]> {
        const dtos = await post_json(`${this.baseUrl}/item/filter`, { filters }) as ItemDto[];
        return dtos.map(d => new Item(d));
    }

    async Add(dto: AddItemDto): Promise<Item> {
        const data = await post_json(`${this.baseUrl}/item`, dto) as ItemDto;
        return new Item(data);
    }

    async Update(item_id: number, dto: UpdateItemDto): Promise<Item> {
        const data = await patch_req(`${this.baseUrl}/item/${item_id}`, dto);
        const json = await (data as Response).json() as ItemDto;
        return new Item(json);
    }

    async Delete(item_id: number) {
        return delete_req(`${this.baseUrl}/item/${item_id}`);
    }

    async AssignType(item_id: number, type_name: string) {
        return post_req(`${this.baseUrl}/item/${item_id}/assign_type/${encodeURIComponent(type_name)}`, {});
    }

    async UnassignType(item_id: number, type_name: string) {
        return delete_req(`${this.baseUrl}/item/${item_id}/assign_type/${encodeURIComponent(type_name)}`);
    }

    async RebuildLinks(): Promise<{ rebuilt: number }> {
        return await post_json(`${this.baseUrl}/item/rebuild-links`, {}) as { rebuilt: number };
    }

    async GetMostViewed(limit = 30): Promise<MostViewedEntry[]> {
        const dtos = await get_json(`${this.baseUrl}/analysis/most-viewed?limit=${limit}`) as MostViewedItemDto[];
        return dtos.map(d => ({
            item: new Item({ item_id: d.item_id, title: d.title, content: '', attributes: {}, types: [], links: [] }),
            viewCount: d.view_count,
        }));
    }

    ExportPdfUrl(item_id: number): string {
        return `${this.baseUrl}/item/id/${item_id}/export/pdf`;
    }

    ExportDocxUrl(item_id: number): string {
        return `${this.baseUrl}/item/id/${item_id}/export/docx`;
    }
}
