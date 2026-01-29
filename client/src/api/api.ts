import AuthAPI from "./auth";
import ItemAPI from "./item";
import MediaAPI from "./media";
import PlannerAPI from "./planner";
import RepeatAPI from "./repeat";
import CommentsAPI from "./comments";

const API = {
    item: ItemAPI,
    auth: AuthAPI,
    planner: PlannerAPI,
    media: MediaAPI,
    repeat: RepeatAPI,
    comments: CommentsAPI,
}

export default API;
