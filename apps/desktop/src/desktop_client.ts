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
    setOpenInNewTabHandler,
    setTabHistoryController,
} from '@websoil/engine';
import { AddItemModal, CommandPaletteModal, ItemSearchModal, RightSidebar, default_side_button_entries, SideButtons } from '@zealot/ui';
import '@zealot/ui/src/shell/mobile_title_bar';
import { DesktopNavigator } from './desktop_navigator';
import './desktop_tool_host';
import { DesktopToolHost } from './desktop_tool_host';
import { getAPI } from './desktop_core';
import './desktop_dock_host';
import { DesktopDockHost, setDockHost, getDockHost } from './desktop_dock_host';

class DesktopClient extends BaseElementEmpty {
    private unlistenMenu: (() => void) | null = null;

    render() {
        const nav = new DesktopNavigator();
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
            const items = await getAPI().Item.GetRandom(1);
            if (items[0]) getNavigator().openItemById(items[0].ItemID);
        });
        commands.runner.register('New Tab', [new Hotkey('t', [CTRL_OR_META_KEY])], () => getDockHost().openInNewTab('/'));
        commands.runner.register('Close Tab', [new Hotkey('w', [CTRL_OR_META_KEY])], () => getDockHost().closeActiveTab());
        commands.runner.register('Go Back', [new Hotkey('[', [CTRL_OR_META_KEY])], () => nav.goBack());
        commands.runner.register('Go Forward', [new Hotkey(']', [CTRL_OR_META_KEY])], () => nav.goForward());

        // The native menu forwards its selections here, so menu actions and
        // in-app hotkeys always use the same command implementations.
        void import('@tauri-apps/api/event').then(({ listen }) =>
            listen<string>('zealot-menu-command', ({ payload }) => {
                const menuCommands: Record<string, string> = {
                    'new-item': ModalCommands.newItem,
                    'new-tab': 'New Tab',
                    'close-tab': 'Close Tab',
                    'today-note': NavigationCommands.openTodayNote,
                    'random-item': NavigationCommands.openRandomItem,
                    'go-back': 'Go Back',
                    'go-forward': 'Go Forward',
                    'go-home': NavigationCommands.goHome,
                    'planner-daily': NavigationCommands.openDailyPlanner,
                    'planner-weekly': NavigationCommands.openWeeklyPlanner,
                    'planner-monthly': NavigationCommands.openMonthlyPlanner,
                    'planner-annual': NavigationCommands.openAnnualPlanner,
                    'time-blocks-day': 'Time Blocks: Today',
                    'time-blocks-week': 'Time Blocks: This Week',
                    'open-media': NavigationCommands.openMedia,
                    'open-analysis': NavigationCommands.openAnalysis,
                    'analysis-recent': 'Analysis: Recent',
                    'open-rules': NavigationCommands.openRules,
                    'open-types': NavigationCommands.openTypes,
                    'open-settings': NavigationCommands.openSettings,
                    'global-search': ModalCommands.openGlobalSearch,
                    'command-palette': ModalCommands.openCommandRunner,
                };
                const command = menuCommands[payload];
                if (command) commands.runner.run(command);
            }).then((unlisten) => {
                this.unlistenMenu = unlisten;
            }),
        );

        this.innerHTML = `
        <header-bar></header-bar>
        <main id="main_web_ui">
            <side-buttons></side-buttons>
            <desktop-tool-host id="left_tool_host"></desktop-tool-host>
            <div class="panel-resize-handle" id="left_resize_handle"></div>
            <desktop-dock-host></desktop-dock-host>
            <div class="panel-resize-handle" id="right_resize_handle"></div>
            <right-sidebar></right-sidebar>
        </main>
        `;

        (document.querySelector('side-buttons')! as SideButtons).init(default_side_button_entries());
        setToolHost(document.querySelector('desktop-tool-host')! as DesktopToolHost);
        setRightSidebarHost(document.querySelector('right-sidebar')! as RightSidebar);

        const dockHost = document.querySelector('desktop-dock-host')! as DesktopDockHost;
        setDockHost(dockHost);
        dockHost.init(nav);

        setOpenInNewTabHandler((path) => getDockHost().openInNewTab(path));

        setTabHistoryController({
            canGoBack: () => getDockHost().canGoBack(),
            canGoForward: () => getDockHost().canGoForward(),
            goBack: () => nav.goBack(),
            goForward: () => nav.goForward(),
            subscribe: (cb) => getDockHost().onHistoryChange(cb),
        });

        initPanelResizers();

    }

    disconnectedCallback(): void {
        this.unlistenMenu?.();
        this.unlistenMenu = null;
    }
}

customElements.define('zealot-desktop-client', DesktopClient);

export { DesktopClient };

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
