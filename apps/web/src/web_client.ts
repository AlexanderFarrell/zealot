import {
    BaseElementEmpty,
    CTRL_OR_META_KEY,
    Hotkey,
    ModalCommands,
    NavigationCommands,
    commands,
    getNavigator,
    registerNavigationCommands,
    registerToolCommands,
    setNavigator,
    setRightSidebarHost,
    setToolHost,
} from "@websoil/engine";
import { API } from "./core";
import { AddItemModal, CommandPaletteModal, ItemSearchModal, RightSidebar, default_side_button_entries, SideButtons } from "@zealot/ui";
import "@zealot/ui/src/shell/mobile_title_bar";
import { WebNavigator } from "./web_navigator";
import "./web_tool_host";
import { WebToolHost } from "./web_tool_host";

class ZealotWebClient extends BaseElementEmpty {
    render() {
        const nav = new WebNavigator();
        setNavigator(nav);
        commands.runner.clear();
        registerNavigationCommands();
        registerToolCommands();
        commands.runner.register(ModalCommands.newItem, [new Hotkey('n', [CTRL_OR_META_KEY])], () => {
            AddItemModal.show();
        });
        commands.runner.register(ModalCommands.openGlobalSearch, [new Hotkey('o', [CTRL_OR_META_KEY])], () => {
            ItemSearchModal.show();
        });
        commands.runner.register(ModalCommands.openCommandRunner, [new Hotkey('p', [CTRL_OR_META_KEY])], () => {
            CommandPaletteModal.show();
        });
        commands.runner.register(NavigationCommands.openRandomItem, [], async () => {
            const items = await API.Item.GetRandom(1);
            if (items[0]) getNavigator().openItemById(items[0].ItemID);
        });

        this.innerHTML = `
        <header-bar class="desktop_only"></header-bar>
        <mobile-title-bar class="mobile_only"></mobile-title-bar>
        <main id="main_web_ui">
            <side-buttons class="desktop_only"></side-buttons>
            <web-tool-host id="left_tool_host" class="desktop_only"></web-tool-host>
            <div class="panel-resize-handle desktop_only" id="left_resize_handle"></div>
            <center-content></center-content>
            <div class="panel-resize-handle desktop_only" id="right_resize_handle"></div>
            <right-sidebar></right-sidebar>
        </main>
        <footer-bar></footer-bar>
        `;

        (document.querySelector("side-buttons")! as SideButtons).init(default_side_button_entries());
        setToolHost(document.querySelector('web-tool-host')! as WebToolHost);
        setRightSidebarHost(document.querySelector('right-sidebar')! as RightSidebar);

        initPanelResizers();

        nav.resolve();
    }
}

customElements.define('zealot-web-client', ZealotWebClient);

export default ZealotWebClient;

const LEFT_W_KEY  = 'zealot:left-panel-w';
const RIGHT_W_KEY = 'zealot:right-panel-w';
const LEFT_W_DEFAULT  = 280;
const RIGHT_W_DEFAULT = 260;

function initPanelResizers(): void {
    const main = document.getElementById('main_web_ui')!;

    const leftW  = Number(localStorage.getItem(LEFT_W_KEY))  || LEFT_W_DEFAULT;
    const rightW = Number(localStorage.getItem(RIGHT_W_KEY)) || RIGHT_W_DEFAULT;
    main.style.setProperty('--left-panel-w',  `${leftW}px`);
    main.style.setProperty('--right-panel-w', `${rightW}px`);

    attachResizer(
        document.getElementById('left_resize_handle')!,
        main,
        'left',
        LEFT_W_KEY,
        100,
        600,
    );
    attachResizer(
        document.getElementById('right_resize_handle')!,
        main,
        'right',
        RIGHT_W_KEY,
        160,
        500,
    );
}

function attachResizer(
    handle: HTMLElement,
    main: HTMLElement,
    side: 'left' | 'right',
    storageKey: string,
    min: number,
    max: number,
): void {
    if (!handle) return;
    const prop = side === 'left' ? '--left-panel-w' : '--right-panel-w';

    handle.addEventListener('mousedown', (e) => {
        e.preventDefault();
        const startX   = e.clientX;
        const startVal = parseInt(getComputedStyle(main).getPropertyValue(prop)) || 0;

        const onMove = (ev: MouseEvent) => {
            const delta = side === 'left' ? ev.clientX - startX : startX - ev.clientX;
            const next  = Math.min(max, Math.max(min, startVal + delta));
            main.style.setProperty(prop, `${next}px`);
        };

        const onUp = (ev: MouseEvent) => {
            const delta = side === 'left' ? ev.clientX - startX : startX - ev.clientX;
            const next  = Math.min(max, Math.max(min, startVal + delta));
            localStorage.setItem(storageKey, String(next));
            document.removeEventListener('mousemove', onMove);
            document.removeEventListener('mouseup',   onUp);
            document.body.style.cursor = '';
            document.body.style.userSelect = '';
        };

        document.body.style.cursor     = 'col-resize';
        document.body.style.userSelect = 'none';
        document.addEventListener('mousemove', onMove);
        document.addEventListener('mouseup',   onUp);
    });
}
