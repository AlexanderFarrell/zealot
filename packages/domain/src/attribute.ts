export class AttributeKind {
    public readonly KindID: number;
    public readonly IsSystem: boolean;
    public Key: string;
    public Description: string;
    public BaseType: AttributeBaseType;
    public Config: AttributeConfig;

    public constructor(dto: AttributeKindDto) {
        this.KindID = dto.kind_id;
        this.IsSystem = dto.is_system;
        this.Key = dto.key;
        this.Description = dto.description;
        if (!(dto.base_type in AttributeBaseTypesArray)) {
            throw new Error("Invalid base type for attribute kind: " + dto.base_type)
        }
        this.BaseType = dto.base_type as AttributeBaseType;
        this.Config = dto.config;
    }
}

export const AttributeBaseTypesArray = [
    'text',
    'integer',
    'decimal',
    'date',
    'week',
    'dropdown',
    'boolean',
    'list'
]

export type AttributeBaseType = typeof AttributeBaseTypesArray[number];

export interface AttributeConfig {
    min?: number;
    max?: number;
    min_len?: number;
    max_len?: number;
    pattern?: string;
    values?: string[];
    list_type?: AttributeBaseType
}

export interface AttributeKindDto {
    kind_id: number,
    key: string,
    description: string,
    base_type: string,
    config: AttributeConfig,
    is_system: boolean,
}

export interface AddAttributeKindDto {
    key: string;
    description: string;
    base_type: AttributeBaseType;
    config: AttributeConfig;
}

export interface UpdateAttributeKind {
    kind_id: number;
    key?: string;
    description?: string;
    base_type?: AttributeBaseType;
    config?: AttributeConfig;
}