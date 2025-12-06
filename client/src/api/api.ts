import AuthAPI from "./auth";
import ItemAPI from "./item";
import PlannerAPI from "./planner";

const API = {
    item: ItemAPI,
    auth: AuthAPI,
    planner: PlannerAPI,
}

export default API;