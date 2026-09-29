import { DateTime } from "luxon";
import { Item, type ItemDto } from "./item";


export const RepeatStatus = [
    'Complete',
    'Skip',
    'Alternate',
    'Not Complete'
]

export type RepeatStatusType = typeof RepeatStatus[number];

export class RepeatEntry {
    public Status: RepeatStatusType;
    public Item: Item;
    public Date: DateTime;
    public Comment: string;

    public constructor(dto: RepeatEntryDto) {
        this.Status = dto.status;
        this.Item = new Item(dto.item);
        this.Date = DateTime.fromISO(dto.date);
        this.Comment = dto.comment;
    }
}

export interface RepeatEntryDto {
    status: string;
    item: ItemDto;
    date: string;
    comment: string;
}

export interface UpdateRepeatEntryDto {
    item_id: number;
    date: DateTime;
    status?: RepeatStatusType;
    comment?: string;
}