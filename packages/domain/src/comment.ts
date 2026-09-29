import { Item, type ItemDto } from "./item";
import { DateTime } from "luxon";

const COMMENT_TIMESTAMP_FORMAT = 'yyyy-MM-dd HH:mm:ss';

export class Comment {
    public readonly CommentID: number;
    public Item: Item;
    public Timestamp: DateTime;
    public Content: string;

    public constructor(dto: CommentDto) {
        this.CommentID = dto.comment_id;
        this.Item = new Item(dto.item);
        this.Timestamp = parseCommentTimestamp(dto.timestamp);
        this.Content = dto.content;
    }
}

export function parseCommentTimestamp(value: string): DateTime {
    const parsedSql = DateTime.fromSQL(value);
    if (parsedSql.isValid) {
        return parsedSql;
    }

    const parsedIso = DateTime.fromISO(value);
    if (parsedIso.isValid) {
        return parsedIso;
    }

    const parsedFormat = DateTime.fromFormat(value, COMMENT_TIMESTAMP_FORMAT);
    if (parsedFormat.isValid) {
        return parsedFormat;
    }

    return DateTime.invalid(`Invalid comment timestamp: ${value}`);
}

export function formatCommentTimestamp(value: DateTime): string {
    return value.toFormat(COMMENT_TIMESTAMP_FORMAT);
}

export interface CommentDto {
	comment_id: number;
	item: ItemDto;
	timestamp: string;
	content: string;
}

export interface AddCommentDto {
    item_id: number;
    timestamp: DateTime;
    content: string;
}

export interface UpdateCommentDto {
    comment_id: number;
    item_id?: number;
    timestamp?: DateTime;
    content?: string;
}
