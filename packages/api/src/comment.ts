import { delete_req, get_json, patch_json, post_json } from "@websoil/engine";
import { Comment } from "@zealot/domain/src/comment";
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
        const data = await post_json(`${this.baseUrl}/comment/`, dto) as CommentDto;
        return new Comment(data);
    }

    async UpdateEntry(dto: UpdateCommentDto): Promise<Comment> {
        const data = await patch_json(`${this.baseUrl}/comment/${dto.comment_id}`, dto) as CommentDto;
        return new Comment(data);
    }

    async DeleteEntry(comment_id: number) {
        return delete_req(`${this.baseUrl}/comment/${comment_id}`);
    }
}
