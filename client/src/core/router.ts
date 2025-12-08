import Navigo from "navigo";
import ItemScreen from "../screens/item_screen";
import SettingsScreen from "../screens/settings_screen";
import { events } from "./events";
import type Content from "../components/content";
import DailyPlannerScreen from "../screens/day_screen";
import { DateTime } from "luxon";
import WeeklyPlannerScreen from "../screens/week_screen";
import MonthlyPlannerScreen from "../screens/month_screen";
import AnnualPlannerScreen from "../screens/year_screen";

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
        "/item/:title": (params: any) => {
            let item_screen = new ItemScreen();
            content_view!.innerHTML = "";
            content_view!.appendChild(item_screen)
            item_screen.LoadItem(decodeURIComponent(params.data['title']))
        },
        "/media": () => {
            content_view!.innerHTML = "<media-screen></media-screen>";
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

