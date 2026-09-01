import { delete_req, get_json, patch_req, post_json } from '@websoil/engine';
import { Item, type ItemDto } from '@zealot/domain/src/item';
import {
    StatisticEntry,
    type CreateStatisticEntryDto,
    type StatisticDailyPointDto,
    type StatisticEntryDto,
    type StatisticEntryPageDto,
    type StatisticSummaryDto,
    type UpdateStatisticEntryDto,
} from '@zealot/domain/src/statistic';
import { BaseAPI } from './common';

export interface StatisticEntryPage {
    count: number;
    nextOffset: number | null;
    entries: StatisticEntry[];
}

function rangeQuery(start?: string, end?: string): URLSearchParams {
    const params = new URLSearchParams();
    if (start) params.set('start', start);
    if (end) params.set('end', end);
    return params;
}

export class StatisticAPI extends BaseAPI {
    async GetItems(parentId?: number): Promise<Item[]> {
        const query = parentId == null ? '' : `?parent_id=${parentId}`;
        const dtos = await get_json(`${this.baseUrl}/statistic/items${query}`) as ItemDto[];
        return dtos.map(dto => new Item(dto));
    }

    async GetEntries(itemId: number, options: { start?: string; end?: string; limit?: number; offset?: number } = {}): Promise<StatisticEntryPage> {
        const params = rangeQuery(options.start, options.end);
        params.set('limit', String(options.limit ?? 50));
        params.set('offset', String(options.offset ?? 0));
        const dto = await get_json(`${this.baseUrl}/statistic/${itemId}/entries?${params}`) as StatisticEntryPageDto;
        return { count: dto.count, nextOffset: dto.next_offset, entries: dto.entries.map(entry => new StatisticEntry(entry)) };
    }

    async GetDaily(itemId: number, start?: string, end?: string): Promise<StatisticDailyPointDto[]> {
        const params = rangeQuery(start, end).toString();
        return await get_json(`${this.baseUrl}/statistic/${itemId}/daily${params ? `?${params}` : ''}`) as StatisticDailyPointDto[];
    }

    async GetSummary(itemId: number, start?: string, end?: string): Promise<StatisticSummaryDto> {
        const params = rangeQuery(start, end).toString();
        return await get_json(`${this.baseUrl}/statistic/${itemId}/summary${params ? `?${params}` : ''}`) as StatisticSummaryDto;
    }

    async Create(itemId: number, dto: CreateStatisticEntryDto): Promise<StatisticEntry> {
        const result = await post_json(`${this.baseUrl}/statistic/${itemId}/entries`, dto) as StatisticEntryDto;
        return new StatisticEntry(result);
    }

    async Update(statisticEntryId: number, dto: UpdateStatisticEntryDto): Promise<StatisticEntry> {
        const response = await patch_req(`${this.baseUrl}/statistic/entries/${statisticEntryId}`, dto) as Response;
        return new StatisticEntry(await response.json() as StatisticEntryDto);
    }

    async Delete(statisticEntryId: number): Promise<void> {
        await delete_req(`${this.baseUrl}/statistic/entries/${statisticEntryId}`);
    }
}

