import { get_json } from "../core/api_helper";
import { ToLocaleISOString } from "../core/time_helper";
import type { Item } from "./item";
import {DateTime} from "luxon";

class PlannerAPIHandler {
    async get_items_on_day(date: DateTime): Promise<Item[]> {
        const iso = date.toISODate();
        return get_json(`/api/planner/day/${iso}`);
    }
}

const PlannerAPI = new PlannerAPIHandler();

export default PlannerAPI;