import { DateTime } from 'luxon';
import { Item, type ItemDto } from './item';

export interface TimeBlockDto {
    block_id:  number;
    item:      ItemDto;
    date:      string;
    start_min: number;
    end_min:   number;
    note:      string;
}

export interface CreateTimeBlockDto {
    item_id:   number;
    date:      string;
    start_min: number;
    end_min:   number;
    note?:     string;
}

export interface UpdateTimeBlockDto {
    block_id:   number;
    date?:      string;
    start_min?: number;
    end_min?:   number;
    note?:      string;
}

function minToLabel(min: number): string {
    const h = Math.floor(min / 60).toString().padStart(2, '0');
    const m = (min % 60).toString().padStart(2, '0');
    return `${h}:${m}`;
}

export class TimeBlock {
    public BlockId:  number;
    public Item:     Item;
    public Date:     DateTime;
    public StartMin: number;
    public EndMin:   number;
    public Note:     string;

    public constructor(dto: TimeBlockDto) {
        this.BlockId  = dto.block_id;
        this.Item     = new Item(dto.item);
        this.Date     = DateTime.fromISO(dto.date);
        this.StartMin = dto.start_min;
        this.EndMin   = dto.end_min;
        this.Note     = dto.note;
    }

    public startLabel(): string {
        return minToLabel(this.StartMin);
    }

    public endLabel(): string {
        return minToLabel(this.EndMin);
    }
}
