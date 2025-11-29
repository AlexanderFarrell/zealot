import Navigo from "navigo";
import { switch_item_to } from "../screens/item_screen";
import SettingsScreen from "../screens/settings_screen";
import { events } from "./events";

export const router = new Navigo("/")


export function setup_router() {
    router.on({
        "/": () => {
            switch_item_to('Home');
        },
        "/item/:title": (params: any) => {
            switch_item_to(params.data['title'])
        },
        "/media": () => {
            document.querySelector('content-')!.innerHTML = "<media-screen></media-screen>";
        },
        "/planner/daily": () => {
            document.querySelector('content-')!.innerHTML = "<daily-planner-screen></daily-planner-screen>";
        },
        "/planner/weekly": () => {
            document.querySelector('content-')!.innerHTML = "<weekly-planner-screen></weekly-planner-screen>";
        },
        "/planner/monthly": () => {
            document.querySelector('content-')!.innerHTML = "<monthly-planner-screen></monthly-planner-screen>";
        },
        "/planner/annual": () => {
            document.querySelector('content-')!.innerHTML = "<annual-planner-screen></annual-planner-screen>";
        },
        "/analysis": () => {
            document.querySelector('content-')!.innerHTML = "<analysis-screen></analysis-screen>";
        },
        "/rules": () => {
            document.querySelector('content-')!.innerHTML = "<rules-screen></rules-screen>";
        },
        "/settings": () => {
            router.navigate('/settings/types')
        },
        "/settings/:screen": (params: any) => {
            let settings_screen = new SettingsScreen();
            let content = document.querySelector('content-')!
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

