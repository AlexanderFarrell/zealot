import Navigo from "navigo";
import ItemScreen from "../item/item_screen";
import SettingsScreen from "../settings/settings_screen";
import Content from "../../app/shell/content";
import DailyPlannerScreen from "../planner/day_screen";
import { DateTime } from "luxon";
import WeeklyPlannerScreen from "../planner/week_screen";
import MonthlyPlannerScreen from "../planner/month_screen";
import AnnualPlannerScreen from "../planner/year_screen";
import TypeScreen from "../types/type_screen";
import TypesScreen from "../types/types_screen";
import { get_item_types, get_type_for_name } from "../../api/item_type";
import MediaScreen from "../media/media_screen";

export const router = new Navigo("/")

export function setup_router() {
    const content_view = document.querySelector('content-')! as Content;
    router.on({
        "/": () => {
            let item_screen = new ItemScreen();
            content_view!.innerHTML = "";
            content_view!.appendChild(item_screen)
            item_screen.LoadItem('Home');
        },
        "/item_id/:item_id": (params: any) => {
            let item_screen = new ItemScreen()
            content_view!.innerHTML = "";
            content_view!.appendChild(item_screen)
            item_screen.LoadItemByID(parseInt(params.data['item_id']))
        },
        "/item/:title": (params: any) => {
            let item_screen = new ItemScreen();
            content_view!.innerHTML = "";
            content_view!.appendChild(item_screen)
            item_screen.LoadItem(decodeURIComponent(params.data['title']))
        },
        "/media/*": (params: any) => {
            content_view!.innerHTML = "";
            content_view!.appendChild(new MediaScreen().init(params.url.substring(6)))
            // content_view!.innerHTML = "<media-screen></media-screen>";
        },
        "/planner/daily": () => {
            let date = DateTime.local();
            router.navigate(`/planner/daily/${date.toISODate()}`)
        },
        "/planner/daily/:date": (params: any) => {
            let date_str = params.data['date'];
            let daily_screen = new DailyPlannerScreen().init(
                DateTime.fromFormat(date_str, "yyyy-MM-dd")
            );
            content_view.innerHTML = "";
            content_view!.appendChild(daily_screen);
        },
        "/planner/weekly": () => {
            let date = DateTime.local();
            router.navigate(`/planner/weekly/${date.toISOWeekDate().substring(0, 8)}`)
        },
        "/planner/weekly/:date": (params: any) => {
            let date_str = params.data['date'];
            let weekly_screen = new WeeklyPlannerScreen().init(
                DateTime.fromObject({
                    weekYear: parseInt(date_str.substring(0, 4)),
                    weekNumber: parseInt(date_str.substring(6, 8))
                })
            );
            content_view!.innerHTML = "";
            content_view!.appendChild(weekly_screen);
        },
        "/planner/monthly": () => {
            let date = DateTime.local();
            router.navigate(`/planner/monthly/${date.toFormat('yyyy-MM')}`)
        },
        "/planner/monthly/:date": (params: any) => {
            let date_str = params.data['date'];
            let date = DateTime.fromFormat(date_str, `yyyy-MM`)
            content_view.innerHTML = "";
            content_view.appendChild(new MonthlyPlannerScreen().init(date));
        },
        "/planner/annual": () => {
            router.navigate(`/planner/annual/${DateTime.now().year}`)
        },
        "/planner/annual/:year": (params: any) => {
            let year = params.data['year'];
            content_view.innerHTML = "";
            content_view.appendChild(new AnnualPlannerScreen().init(DateTime.fromObject({
                year: year
            })))
        },
        "/types": () => {
            let content = content_view!
            content.innerHTML = "";
            content.appendChild(new TypesScreen)
        },
        "/types/:title": async (params: any) => {
            let title = params.data['title']
            try {
                let type = await get_type_for_name(title);
                if (type != null) {
                    content_view.innerHTML = ""
                    content_view.appendChild(new TypeScreen().init(type))
                } else {
                    content_view.innerHTML = "Type not found: " + title
                }
            } catch (e) {
                console.error(e);
                content_view.innerHTML = "Error getting type " + title;
            }
        },
        "/analysis": () => {
            content_view!.innerHTML = "<analysis-screen></analysis-screen>";
        },
        "/rules": () => {
            content_view!.innerHTML = "<rules-screen></rules-screen>";
        },
        "/settings": () => {
            router.navigate('/settings/attributes')
        },
        "/settings/:screen": (params: any) => {
            let settings_screen = new SettingsScreen();
            let content = content_view!
            content.innerHTML = "";
            content.appendChild(settings_screen);
            settings_screen.switch_screen(params.data['screen'])
        }
    })
    router.resolve();
}

export interface Route {
    path: string;
    name?: string;
    component: (match: any) => HTMLElement;
}

const ROOT_SELECTOR = "content-";

function mount(component: (match: any) => HTMLElement, match: any) {
    const root = document.querySelector(ROOT_SELECTOR);
    if (!root) throw new Error(`No root element ${ROOT_SELECTOR}`);
    root.innerHTML = "";
    const el = component(match);
    root.appendChild(el);
}

