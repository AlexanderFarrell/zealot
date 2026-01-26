import AuthAPI from "./auth";
import ItemAPI from "./item";
import MediaAPI from "./media";
import PlannerAPI from "./planner";
import RepeatAPI from "./repeat";

const API = {
    item: ItemAPI,
    auth: AuthAPI,
    planner: PlannerAPI,
    media: MediaAPI,
    repeat: RepeatAPI,
}

export default API;
