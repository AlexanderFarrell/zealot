import { delete_req, get_json, patch_req } from "@websoil/engine";
import { Comment, CommentDto, UpdateCommentDto } from "@zealot/domain/src/comment";
import { DateTime } from "luxon";
import { BaseAPI } from "./common";

export class CommentAPI extends BaseAPI {
    async GetForDay(date: DateTime): Promise<Comment[]> {
        const iso = date.toISODate();
        let dtos: Array<CommentDto> = await get_json(`${this.baseUrl}/comments/day/${iso}`);
        return dtos.map(d => new Comment(d));
    }

    async GetForItem(item_id: number): Promise<Comment[]> {
        let dtos: Array<CommentDto> = await get_json(`${this.baseUrl}/comments/item/${item_id}`)
        return dtos.map(d => new Comment(d));
    }

    async UpdateEntry(dto: UpdateCommentDto) {
        return patch_req(`${this.baseUrl}/comments/${dto.comment_id}`, dto)
    }

    async DeleteEntry(comment_id: number) {
        return delete_req(`${this.baseUrl}/comments/${comment_id}`);
    }
}