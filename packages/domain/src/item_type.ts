
export class ItemType {
    public readonly TypeID: number;
    public readonly IsSystem: boolean;
    public Name: string;
    public Description: string;
    public RequiredAttributes: string[];

    public constructor(dto: ItemTypeDto) {
        this.TypeID = dto.type_id;
        this.IsSystem = dto.is_system;
        this.Name = dto.name;
        this.Description = dto.description;
        this.RequiredAttributes = dto.required_attributes;
    }
}

export class ItemTypeRef {
    public readonly TypeID: number;
    public readonly IsSystem: boolean;
    public Name: string;

    public constructor(dto: ItemTypeRefDto) {
        this.TypeID = dto.type_id;
        this.IsSystem = dto.is_system;
        this.Name = dto.name;
    }
}

export interface ItemTypeDto {
    type_id: number;
    name: string;
    description: string;
    is_system: boolean;
    required_attributes: string[];
}

export interface ItemTypeRefDto {
    type_id: number;
    name: string;
    is_system: boolean;
}

export interface AddItemTypeDto {
    name: string;
    description: string;
    required_attributes: string[];
}

export interface UpdateItemTypeDto {
    type_id: number;
    name?: string;
    description?: string;
    required_attributes?: string[];
}
