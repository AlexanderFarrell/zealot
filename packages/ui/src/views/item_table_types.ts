import type { Item, ItemRelationship } from '@zealot/domain/src/item';

export type ItemTableColumn =
    | {
        kind: 'title';
        editable?: boolean;
        label?: string;
        sortable?: boolean;
    }
    | {
        kind: 'types';
        editable?: boolean;
        label?: string;
        sortable?: boolean;
    }
    | {
        attributeKey: string;
        kind: 'attribute';
        editable?: boolean;
        label?: string;
        sortable?: boolean;
    };

export interface ItemTableCreateRowConfig {
    contextItemId?: number;
    defaultAttributes?: Record<string, unknown>;
    defaultTypes?: string[];
    enabled: boolean;
    onSuccess?: (item: Item) => void;
    panelMode?: boolean;
    relationship?: ItemRelationship;
    submitLabel?: string;
    typesEditable?: boolean;
}

export interface ItemTableViewConfig {
    columns: ItemTableColumn[];
    createRow?: ItemTableCreateRowConfig;
    emptyMessage?: string;
    items: Item[];
    onOpenItem?: (item: Item) => void;
}

export interface CreateDraftState {
    attributes: Record<string, unknown>;
    title: string;
    types: string[];
}

export type SortDirection = 'asc' | 'desc' | null;
