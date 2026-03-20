import { ItemType, type ItemTypeDto } from "./item_type";

export class Item {
    public readonly ItemID: number;
    public Title: string;
    public Content: string;
    public Attributes: Record<string, any>;
    public Types: Array<ItemType>;
    public Related: Array<Item>;

    public constructor(dto: ItemDto) {
        this.ItemID = dto.item_id;
        this.Title = dto.title;
        this.Content = dto.content;
        this.Attributes = dto.attributes || {};
        this.Types = dto.types?.map(r => {
            return new ItemType(r)
        }) || [];
        this.Related = dto.related?.map(r => {
            return new Item(r);
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

    public get Children(): Array<Item> {
        return this.WhereThisIsA('Parent');
    }

    public WhereThisIsA(relation: string): Array<Item> {
        return this.Related.filter(i => {
            if (!(relation in i.Attributes)) {
                return false;
            }
            if (!Array.isArray(i.Attributes[relation])) {
                return false;
            }
            return this.Title in i.Attributes[relation]
        })
    }
}

export interface ItemDto {
    item_id: number,
    title: string,
    content: string,
    attributes?: Record<string, any>,
    types?: Array<ItemTypeDto>,
    related?: Array<ItemDto>    
}

export interface AddItemDto {
    title: string;
    content?: string;
    attributes?: Record<string, any>;
    types?: Array<String>;
}

export interface UpdateItemDto {
    item_id: number;
    title?: string;
    content?: string;
    attributes?: Record<string, any>
}