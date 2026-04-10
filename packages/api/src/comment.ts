import { delete_req, get_json, patch_json, post_json } from "@websoil/engine";
import { Comment, formatCommentTimestamp } from "@zealot/domain/src/comment";
import type { AddCommentDto, CommentDto, UpdateCommentDto } from "@zealot/domain/src/comment";
import { DateTime } from "luxon";
import { BaseAPI } from "./common";

export class CommentAPI extends BaseAPI {
    async GetForDay(date: DateTime): Promise<Comment[]> {
        const iso = date.toISODate();
        let dtos: Array<CommentDto> = await get_json(`${this.baseUrl}/comment/day/${iso}`);
        return dtos.map(d => new Comment(d));
    }

    async GetForItem(item_id: number): Promise<Comment[]> {
        let dtos: Array<CommentDto> = await get_json(`${this.baseUrl}/comment/item/${item_id}`)
        return dtos.map(d => new Comment(d));
    }

    async AddComment(dto: AddCommentDto): Promise<Comment> {
        const data = await post_json(`${this.baseUrl}/comment/`, {
            content: dto.content,
            item_id: dto.item_id,
            timestamp: formatCommentTimestamp(dto.timestamp),
        }) as CommentDto;
        return new Comment(data);
    }

    async UpdateEntry(dto: UpdateCommentDto): Promise<Comment> {
        const payload: {
            comment_id: number;
            content?: string;
            item_id?: number;
            timestamp?: string;
        } = {
            comment_id: dto.comment_id,
        };

        if (dto.content != null) {
            payload.content = dto.content;
        }
        if (dto.item_id != null) {
            payload.item_id = dto.item_id;
        }
        if (dto.timestamp != null) {
            payload.timestamp = formatCommentTimestamp(dto.timestamp);
        }

        const data = await patch_json(`${this.baseUrl}/comment/${dto.comment_id}`, payload) as CommentDto;
        return new Comment(data);
    }

    async DeleteEntry(comment_id: number) {
        return delete_req(`${this.baseUrl}/comment/${comment_id}`);
    }
}
