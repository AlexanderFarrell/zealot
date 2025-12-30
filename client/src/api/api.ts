import AuthAPI from "./auth";
import ItemAPI from "./item";
import MediaAPI from "./media";
import PlannerAPI from "./planner";

const API = {
    item: ItemAPI,
    auth: AuthAPI,
    planner: PlannerAPI,
    media: MediaAPI,
}

export default API;