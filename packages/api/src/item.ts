import { delete_req, get_json, patch_req, post_json, post_req } from "@websoil/engine";
import { Item } from "@zealot/domain/src/item";
import type { AddItemDto, ItemDto, UpdateItemDto } from "@zealot/domain/src/item";
import { BaseAPI } from "./common";

export interface AttributeFilterDto {
    key: string;
    value: any;
}

export class ItemAPI extends BaseAPI {
    public constructor(baseURL: string) {
        super(baseURL)
    }

    async GetAll(type?: string): Promise<Item[]> {
        const url = type
            ? `${this.baseUrl}/item/?type=${encodeURIComponent(type)}`
            : `${this.baseUrl}/item/`;
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

    async Search(term: string): Promise<Item[]> {
        const dtos: ItemDto[] = await get_json(`${this.baseUrl}/item/search?term=${encodeURIComponent(term)}`);
        return dtos.map(d => new Item(d));
    }

    async GetChildren(item_id: number): Promise<Item[]> {
        const dtos: ItemDto[] = await get_json(`${this.baseUrl}/item/children/${item_id}`);
        return dtos.map(d => new Item(d));
    }

    async GetRelated(item_id: number): Promise<Item[]> {
        const dtos: ItemDto[] = await get_json(`${this.baseUrl}/item/related/${item_id}`);
        return dtos.map(d => new Item(d));
    }

    async Filter(filters: AttributeFilterDto[]): Promise<Item[]> {
        const dtos = await post_json(`${this.baseUrl}/item/filter`, { filters }) as ItemDto[];
        return dtos.map(d => new Item(d));
    }

    async Add(dto: AddItemDto): Promise<Item> {
        const data = await post_json(`${this.baseUrl}/item/`, dto) as ItemDto;
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
}
