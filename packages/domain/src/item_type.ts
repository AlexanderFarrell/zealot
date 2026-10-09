
export class ItemType {
    public readonly TypeID: number;
    public readonly IsSystem: boolean;
    public Name: string;
    public Description: string;
    public RequiredAttributes: string[];
    /** Type-level default per required attribute; overrides the attribute kind's default. */
    public RequiredDefaults: Record<string, unknown>;

    public constructor(dto: ItemTypeDto) {
        this.TypeID = dto.type_id;
        this.IsSystem = dto.is_system;
        this.Name = dto.name;
        this.Description = dto.description;
        this.RequiredAttributes = dto.required_attributes;
        this.RequiredDefaults = dto.required_defaults ?? {};
    }
}

/**
 * The value a new `key` starts from for an item with these types: the type-level defaults of the
 * types that require it. None set: the kind default. All agree: that value. Any disagree: the kind
 * default, so there is no type-specific suggestion. Mirrors `effective_default` on the server.
 */
export function effectiveDefault(types: ItemType[], key: string, kindDefault: unknown): unknown {
    const values = types
        .filter((type) => type.RequiredAttributes.includes(key) && key in type.RequiredDefaults)
        .map((type) => type.RequiredDefaults[key]);
    const [first, ...rest] = values;
    if (values.length === 0) return kindDefault;
    const same = JSON.stringify(first);
    return rest.every((value) => JSON.stringify(value) === same) ? first : kindDefault;
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
    /** Type-level default per required attribute, overriding the kind's default. */
    required_defaults?: Record<string, unknown>;
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
    required_defaults?: Record<string, unknown>;
}

export interface UpdateItemTypeDto {
    type_id: number;
    name?: string;
    description?: string;
    required_attributes?: string[];
    /** Set (value) or clear (null) type-level defaults; keys not listed are unchanged. */
    required_defaults?: Record<string, unknown>;
    /** Report what a change to `required_attributes` would do without writing anything. */
    dry_run?: boolean;
}
