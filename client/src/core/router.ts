import Navigo from "navigo";
import ItemScreen from "../screens/item_screen";
import SettingsScreen from "../screens/settings_screen";
import { events } from "./events";
import type Content from "../components/content";

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
            content_view!.innerHTML = "<daily-planner-screen></daily-planner-screen>";
        },
        "/planner/weekly": () => {
            content_view!.innerHTML = "<weekly-planner-screen></weekly-planner-screen>";
        },
        "/planner/monthly": () => {
            content_view!.innerHTML = "<monthly-planner-screen></monthly-planner-screen>";
        },
        "/planner/annual": () => {
            content_view!.innerHTML = "<annual-planner-screen></annual-planner-screen>";
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

