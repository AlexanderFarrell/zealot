import { ItemTypeRef, type ItemTypeRefDto } from "./item_type";

export class ItemLink {
    public readonly OtherItemID: number;
    public Relationship: ItemRelationship;

    public constructor(dto: ItemLinkDto) {
        this.OtherItemID = dto.other_item_id;
        this.Relationship = dto.relationship;
    }
}

export class Item {
    public readonly ItemID: number;
    public Title: string;
    public Content: string;
    public Attributes: Record<string, any>;
    public Types: Array<ItemTypeRef>;
    public Links: Array<ItemLink>;

    public constructor(dto: ItemDto) {
        this.ItemID = dto.item_id;
        this.Title = dto.title;
        this.Content = dto.content;
        this.Attributes = dto.attributes || {};
        this.Types = dto.types?.map(r => {
            return new ItemTypeRef(r)
        }) || [];
        this.Links = dto.links?.map(r => {
            return new ItemLink(r);
        }) || [];
    }

    public get DisplayTitle(): string {
        let title = '';
        if ('Icon' in this.Attributes) {
            title += this.Attributes['Icon'] + " ";
        }
        title += this.Title;
        return title;
    }
}

export interface ItemDto {
    item_id: number,
    title: string,
    content: string,
    attributes?: Record<string, any>,
    types?: Array<ItemTypeRefDto>,
    links?: Array<ItemLinkDto>
}

export type ItemRelationship =
    | 'parent'
    | 'blocks'
    | 'tag'
    | 'topic'
    | 'other';

export interface ItemLinkDto {
    other_item_id: number;
    relationship: ItemRelationship;
}

export interface AddItemDto {
    title: string;
    content?: string;
    attributes?: Record<string, any>;
    types?: Array<string>;
    links?: Array<ItemLinkDto>;
}

export interface UpdateItemDto {
    item_id: number;
    title?: string;
    content?: string;
    attributes?: Record<string, any>;
    links?: Array<ItemLinkDto>;
}
