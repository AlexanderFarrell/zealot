import { DateTime } from "luxon";
import { delete_req, get_json, post_req } from "../shared/api_helper";
import type { Item } from "./item";

export interface TrackerEntry {
	tracker_id: number;
	item: Item;
	timestamp: string;
	level: number;
	comment: string;
}

class TrackerAPIHandler {
	async get_for_day(date: DateTime): Promise<TrackerEntry[]> {
		const iso = date.toISODate();
		return get_json(`/api/tracker/day/${iso}`);
	}

	async add_entry(item_id: number, timestamp: DateTime, level: number, comment: string) {
		return post_req(`/api/tracker`, {
			item_id,
			timestamp: timestamp.toISO(),
			level,
			comment
		});
	}

	async delete_entry(tracker_id: number) {
		return delete_req(`/api/tracker/${tracker_id}`);
	}
}

const TrackerAPI = new TrackerAPIHandler();

export default TrackerAPI;
