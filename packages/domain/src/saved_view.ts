import { Item } from "./item";
import type { ItemDto } from "./item";

export type ViewDisplay = "cards" | "table";

/** The parsed body of a `zealot-view` block. `query` is the item query and stays loosely typed. */
export interface SavedViewDefinition {
    query: Record<string, unknown>;
    group_by?: string;
    columns?: string[];
    display?: ViewDisplay;
    [key: string]: unknown;
}

export interface ViewGroupDto {
    key: unknown;
    label: string;
    item_ids: number[];
}

export interface ViewResultDto {
    view_id?: number;
    title: string;
    total: number;
    offset: number;
    items: ItemDto[];
    group_by?: string;
    groups?: ViewGroupDto[];
    columns?: string[];
    display?: ViewDisplay;
    warnings?: string[];
}

export interface ViewInspectDto {
    view_id?: number;
    title?: string;
    definition: SavedViewDefinition;
    resolved_query: Record<string, unknown>;
    today: string;
    time_zone: string;
    total: number;
    elapsed_ms: number;
    warnings?: string[];
}

export interface ViewGroup {
    key: unknown;
    label: string;
    items: Item[];
}

export class ViewResult {
    public constructor(
        public readonly ViewID: number | null,
        public readonly Title: string,
        public readonly Total: number,
        public readonly Offset: number,
        public readonly Items: Item[],
        public readonly GroupBy: string | null,
        public readonly Groups: ViewGroup[],
        public readonly Columns: string[],
        public readonly Display: ViewDisplay | null,
        public readonly Warnings: string[],
    ) {}

    /** Items past this page that a view's own page can load. */
    public get Remaining(): number {
        return Math.max(0, this.Total - this.Offset - this.Items.length);
    }

    public static fromDto(dto: ViewResultDto): ViewResult {
        const items = dto.items.map((d) => new Item(d));
        const byId = new Map(items.map((item) => [item.ItemID, item]));
        const groups: ViewGroup[] = (dto.groups ?? []).map((group) => ({
            key: group.key,
            label: group.label,
            // an item with several values appears in each group, so look up rather than consume
            items: group.item_ids.flatMap((id) => {
                const item = byId.get(id);
                return item ? [item] : [];
            }),
        }));
        return new ViewResult(
            dto.view_id ?? null,
            dto.title,
            dto.total,
            dto.offset,
            items,
            dto.group_by ?? null,
            groups,
            dto.columns ?? [],
            dto.display ?? null,
            dto.warnings ?? [],
        );
    }
}
