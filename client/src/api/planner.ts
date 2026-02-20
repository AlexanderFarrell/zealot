import { get_json } from "../shared/api_helper";
import type { Item } from "./item";
import {DateTime} from "luxon";

class PlannerAPIHandler {
    async get_items_on_day(date: DateTime): Promise<Item[]> {
        const iso = date.toISODate();
        return get_json(`/api/planner/day/${iso}`);
    }

    async get_items_for_week(date: DateTime): Promise<Item[]> {
        const str = date.toISOWeekDate()!.substring(0, 8);
        return get_json(`/api/planner/week/${str}`)
    }

    async get_items_for_month(date: DateTime): Promise<Item[]> {
        return get_json(`/api/planner/month/${date.month}/year/${date.year}`)
    }

    async get_items_for_year(date: DateTime): Promise<Item[]> {
        return get_json(`/api/planner/year/${date.year}`)
    }
}

const PlannerAPI = new PlannerAPIHandler();

export default PlannerAPI;