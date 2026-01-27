import AuthAPI from "./auth";
import ItemAPI from "./item";
import MediaAPI from "./media";
import PlannerAPI from "./planner";
import RepeatAPI from "./repeat";
import TrackerAPI from "./tracker";

const API = {
    item: ItemAPI,
    auth: AuthAPI,
    planner: PlannerAPI,
    media: MediaAPI,
    repeat: RepeatAPI,
    tracker: TrackerAPI,
}

export default API;
