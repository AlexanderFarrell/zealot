import Navigo from "navigo";

export const router = new Navigo("/", {

})

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

