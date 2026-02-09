import { get_json, patch_json } from "../shared/api_helper";
import type { Item } from "./item";
import { DateTime } from "luxon";

export interface RepeatStatusDate {
	status: string;
	item: Item;
	date: string;
	comment: string;
}

class RepeatAPIHandler {
	async get_for_day(date: DateTime): Promise<RepeatStatusDate[]> {
		const iso = date.toISODate();
		return get_json(`/api/repeat/day/${iso}`);
	}

	async set_status(item_id: number, date: DateTime, status: string, comment: string = "") {
		const iso = date.toISODate();
		return patch_json(`/api/repeat/${item_id}/day/${iso}`, {
			status,
			comment
		});
	}
}

const RepeatAPI = new RepeatAPIHandler();

export default RepeatAPI;
