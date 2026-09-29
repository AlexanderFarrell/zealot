import { DateTime } from 'luxon';
import { BaseAPI } from './common';
import { TimeBlock, type TimeBlockDto, type CreateTimeBlockDto, type UpdateTimeBlockDto } from '@zealot/domain/src/time_block';
import { delete_req, get_json, patch_req, post_json } from '@websoil/engine';

export class TimeBlockAPI extends BaseAPI {
    public async GetForDay(date: DateTime): Promise<TimeBlock[]> {
        const iso = date.toISODate();
        const dtos: TimeBlockDto[] = await get_json(`${this.baseUrl}/time_block/day/${iso}`);
        return dtos.map(d => new TimeBlock(d));
    }

    public async GetForRange(start: DateTime, end: DateTime): Promise<TimeBlock[]> {
        const s = start.toISODate();
        const e = end.toISODate();
        const dtos: TimeBlockDto[] = await get_json(`${this.baseUrl}/time_block/range?start=${s}&end=${e}`);
        return dtos.map(d => new TimeBlock(d));
    }

    public async GetForItem(itemId: number): Promise<TimeBlock[]> {
        const dtos: TimeBlockDto[] = await get_json(`${this.baseUrl}/time_block/item/${itemId}`);
        return dtos.map(d => new TimeBlock(d));
    }

    public async Create(dto: CreateTimeBlockDto): Promise<TimeBlock> {
        const result: TimeBlockDto = await post_json(`${this.baseUrl}/time_block`, dto);
        return new TimeBlock(result);
    }

    public async Update(dto: UpdateTimeBlockDto): Promise<void> {
        await patch_req(`${this.baseUrl}/time_block/${dto.block_id}`, dto);
    }

    public async Delete(blockId: number): Promise<void> {
        await delete_req(`${this.baseUrl}/time_block/${blockId}`);
    }
}
