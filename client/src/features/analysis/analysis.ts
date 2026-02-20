import { DateTime } from "luxon";
import type { AttributeFilter, Item } from "../../api/item";
import API from "../../api/api";

export class Analysis {
	static async get_items_days(days: number) {
        const since = DateTime.now().minus({days: days-1}).toISODate();

        const filters: AttributeFilter[] = [
            {key: "Date", op: "gte", value: since},
        ];

        const items = await API.item.filter(filters);
        return items;
    }

    static group_by_sum(items: Item[], attribute: string): Record<string,number> {
        let categories: Record<string, number> = {}

        items.forEach(item => {
            let val = item.attributes![attribute]
            if (val == null) {
                val = "None";
            }

            if (val != null) {
                if (!(val in categories)) {
                    categories[val] = 1
                } else {
                    categories[val]++;
                }
            }
        })

        return categories
    }

	static compute_total_score(items: Item[]): number {
		let score = 0;
		items.forEach(item => {
			let priority_score = (item.attributes!['Priority'] || 1) * (item.attributes!["AP"] || 1) * 100;
			score += priority_score;
		})
		return score;
	}

	static compute_completed_score(items: Item[]): number {
		let score = 0;
		items.forEach(item => {
			if (item.attributes!['Status'] == null) {
				return;
			}
			let priority_score = (item.attributes!['Priority'] || 1) * (item.attributes!["AP"] || 1) * 100;
			if (item.attributes!['Status'] == 'Complete' || item.attributes!['Status'] == 'Rejected') {
				score += priority_score;
			}
		})
		return score;
	}
}

export default Analysis;