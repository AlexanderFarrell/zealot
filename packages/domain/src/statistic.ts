import { DateTime } from 'luxon';

export interface StatisticEntryDto {
    statistic_entry_id: number;
    item_id: number;
    value: number;
    occurred_at: string;
    related_item_id: number | null;
    comment: string | null;
    created_at: string;
    updated_at: string;
}

export interface CreateStatisticEntryDto {
    value: number;
    occurred_at?: string;
    related_item_id?: number;
    comment?: string;
}

export interface UpdateStatisticEntryDto {
    value?: number;
    occurred_at?: string;
    related_item_id?: number | null;
    comment?: string | null;
}

export interface StatisticEntryPageDto {
    count: number;
    next_offset: number | null;
    entries: StatisticEntryDto[];
}

export interface StatisticDailyPointDto {
    date: string;
    value: number;
    count: number;
}

export interface StatisticSummaryPointDto {
    value: number;
    occurred_at: string;
}

export interface StatisticSummaryDto {
    count: number;
    first: StatisticSummaryPointDto | null;
    latest: StatisticSummaryPointDto | null;
    minimum: number | null;
    maximum: number | null;
    average: number | null;
    sum: number | null;
    delta: number | null;
}

export class StatisticEntry {
    public StatisticEntryID: number;
    public ItemID: number;
    public Value: number;
    public OccurredAt: DateTime;
    public RelatedItemID: number | null;
    public Comment: string | null;
    public CreatedAt: DateTime;
    public UpdatedAt: DateTime;

    public constructor(dto: StatisticEntryDto) {
        this.StatisticEntryID = dto.statistic_entry_id;
        this.ItemID = dto.item_id;
        this.Value = dto.value;
        this.OccurredAt = DateTime.fromISO(dto.occurred_at, { setZone: true });
        this.RelatedItemID = dto.related_item_id;
        this.Comment = dto.comment;
        this.CreatedAt = DateTime.fromISO(dto.created_at, { setZone: true });
        this.UpdatedAt = DateTime.fromISO(dto.updated_at, { setZone: true });
    }
}

