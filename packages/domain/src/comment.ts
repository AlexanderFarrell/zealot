import { Item, type ItemDto } from "./item";
import { DateTime } from "luxon";

export class Comment {
    public readonly CommentID: number;
    public Item: Item;
    public Timestamp: DateTime;
    public Content: string;

    public constructor(dto: CommentDto) {
        this.CommentID = dto.comment_id;
        this.Item = new Item(dto.item);
        this.Timestamp = DateTime.fromISO(dto.timestamp);
        this.Content = dto.content;
    }
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