import { BaseElementEmpty, NavigationCommands, commands, getTabHistoryController } from "@websoil/engine";
import { ZealotIcon, icons } from "@zealot/content";

const RIGHT_COLLAPSED_KEY = 'zealot:right-sidebar-collapsed';
const RIGHT_COLLAPSED_CLASS = 'right-panel-collapsed';

export class HeaderBar extends BaseElementEmpty {
    private unsubscribeHistory: (() => void) | null = null;

    async render() {
        this.classList.add('box');
        this.innerHTML = `
        <button name="home">
            <img src="${ZealotIcon}" alt="Zealot">
            <span>Zealot</span>
        </button>
        <button name="back" class="tauri_only header-bar-icon-btn" title="Back" disabled><img src="${icons.back}" alt="Back"></button>
        <button name="forward" class="tauri_only header-bar-icon-btn" title="Forward" disabled><img src="${icons.forward}" alt="Forward"></button>
        <div class="header-bar-spacer" data-tauri-drag-region></div>
        <button name="toggle-right" class="desktop_only header-bar-icon-btn" title="Toggle right sidebar">&#x25E7;</button>
        <div class="header-bar-window-controls tauri_only">
            <button name="minimise" title="Minimise">&#x2013;</button>
            <button name="maximise" title="Maximise">&#x25A1;</button>
            <button name="close" title="Close">&#x2715;</button>
        </div>
        `;

        this.querySelector<HTMLButtonElement>('[name="home"]')!
            .addEventListener('click', () => commands.runner.run(NavigationCommands.goHome));

        const backBtn = this.querySelector<HTMLButtonElement>('[name="back"]')!;
        const forwardBtn = this.querySelector<HTMLButtonElement>('[name="forward"]')!;
        backBtn.addEventListener('click', () => getTabHistoryController()?.goBack());
        forwardBtn.addEventListener('click', () => getTabHistoryController()?.goForward());

        // setTabHistoryController() is called later in the same synchronous render pass
        // that mounts this element, so defer until that's had a chance to run.
        queueMicrotask(() => {
            const controller = getTabHistoryController();
            if (!controller) return;
            const updateHistoryButtons = () => {
                backBtn.disabled = !controller.canGoBack();
                forwardBtn.disabled = !controller.canGoForward();
            };
            updateHistoryButtons();
            this.unsubscribeHistory = controller.subscribe(updateHistoryButtons);
        });

        const main = document.getElementById('main_web_ui');
        const toggleRightBtn = this.querySelector<HTMLButtonElement>('[name="toggle-right"]')!;

        const applyCollapsed = (collapsed: boolean) => {
            main?.classList.toggle(RIGHT_COLLAPSED_CLASS, collapsed);
            toggleRightBtn.innerHTML = collapsed ? '&#x25E8;' : '&#x25E7;';
        };

        const collapsed = localStorage.getItem(RIGHT_COLLAPSED_KEY) === 'true';
        applyCollapsed(collapsed);

        toggleRightBtn.addEventListener('click', () => {
            const next = !(main?.classList.contains(RIGHT_COLLAPSED_CLASS) ?? false);
            localStorage.setItem(RIGHT_COLLAPSED_KEY, String(next));
            applyCollapsed(next);
        });

        if (document.body.classList.contains('tauri-app')) {
            const { getCurrentWindow } = await import('@tauri-apps/api/window');
            const win = getCurrentWindow();

            this.querySelector('[name="minimise"]')!.addEventListener('click', () => { void win.minimize(); });
            this.querySelector('[name="maximise"]')!.addEventListener('click', () => { void win.toggleMaximize(); });
            this.querySelector('[name="close"]')!.addEventListener('click', () => { void win.close(); });

            this.querySelector('.header-bar-spacer')!.addEventListener('dblclick', () => { void win.toggleMaximize(); });
        }
    }

    disconnectedCallback(): void {
        this.unsubscribeHistory?.();
        this.unsubscribeHistory = null;
    }
}

customElements.define('header-bar', HeaderBar);
