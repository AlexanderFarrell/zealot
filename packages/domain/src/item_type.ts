
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

export class ItemTypeSummary {
    public readonly TypeID: number;
    public readonly IsSystem: boolean;
    public readonly Name: string;
    public readonly RequiredAttributesCount: number;
    public readonly ItemCount: number;

    public constructor(dto: ItemTypeSummaryDto) {
        this.TypeID = dto.type_id;
        this.IsSystem = dto.is_system;
        this.Name = dto.name;
        this.RequiredAttributesCount = dto.required_attributes_count;
        this.ItemCount = dto.item_count;
    }
}

export interface ItemTypeDto {
    type_id: number;
    name: string;
    description: string;
    is_system: boolean;
    required_attributes: string[];
    /** Only on the response to a requirement change (including a dry run). */
    report?: RequiredChangeReport;
}

export interface RequiredChangeReport {
    dry_run: boolean;
    added: string[];
    removed: string[];
    /** Items given a default value, per added attribute key. */
    backfilled: Record<string, number>;
    /** Items lacking an added attribute that has no default; non-empty means the change is rejected. */
    non_compliant: {
        total: number;
        by_key: Record<string, number>;
        sample_ids: Record<string, number[]>;
    };
}

export interface NonCompliantItemDto {
    item_id: number;
    title: string;
    archived: boolean;
    missing: string[];
}

export interface NonCompliantPageDto {
    total: number;
    items: NonCompliantItemDto[];
}

export interface ItemTypeRefDto {
    type_id: number;
    name: string;
    is_system: boolean;
}

export interface ItemTypeSummaryDto {
    type_id: number;
    name: string;
    is_system: boolean;
    required_attributes_count: number;
    item_count: number;
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
    /** Report what a change to `required_attributes` would do without writing anything. */
    dry_run?: boolean;
}
