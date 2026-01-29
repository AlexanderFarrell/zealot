import { get_json, post_req, delete_req, patch_req } from "../core/api_helper";
import type { Item } from "./item";
import { DateTime } from "luxon";

export interface CommentEntry {
	comment_id: number;
	item: Item;
	timestamp: string;
	content: string;
}

class CommentsAPIHandler {
	async get_for_day(date: DateTime): Promise<CommentEntry[]> {
		const iso = date.toISODate();
		return get_json(`/api/comments/day/${iso}`);
	}

	async get_for_item(item_id: number): Promise<CommentEntry[]> {
		return get_json(`/api/comments/item/${item_id}`);
	}

	async add_entry(item_id: number, timestamp: DateTime, content: string) {
		return post_req(`/api/comments`, {
			item_id,
			timestamp: timestamp.toISO(),
			content
		});
	}

	async update_entry(comment_id: number, content: string, timestamp?: DateTime) {
		return patch_req(`/api/comments/${comment_id}`, {
			content,
			time: timestamp ? timestamp.toISO() : ""
		});
	}

	async delete_entry(comment_id: number) {
		return delete_req(`/api/comments/${comment_id}`);
	}
}

const CommentsAPI = new CommentsAPIHandler();

export default CommentsAPI;
