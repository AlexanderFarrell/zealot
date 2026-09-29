import type { ToolHost, ToolShowOptions, ToolView } from '@websoil/engine';
import { CalendarToolView, NavTreeToolView, RandomToolView, SearchToolView } from '@zealot/ui';
import { API } from './core';

const LEFT_COLLAPSED_CLASS = 'left-panel-collapsed';

export class WebToolHost extends HTMLElement implements ToolHost {
    private activeView: ToolView = 'nav_tree';
    private readonly views = new Map<ToolView, HTMLElement>();
    private rendered = false;

    connectedCallback(): void {
        if (!this.rendered) {
            this.render();
        }
    }

    show(view: ToolView, options?: ToolShowOptions): void {
        this.activeView = view;
        if (!this.rendered) {
            this.render();
        }
        this._setCollapsed(false);
        this.updateVisibility();
        if (view === 'search' && options?.focus) {
            queueMicrotask(() => {
                (this.views.get('search') as SearchToolView | undefined)?.focusInput();
            });
        }
    }

    toggle(view: ToolView, options?: ToolShowOptions): void {
        if (!this.rendered) {
            this.render();
        }
        const main = document.getElementById('main_web_ui');
        const isCollapsed = main?.classList.contains(LEFT_COLLAPSED_CLASS) ?? false;
        if (this.activeView === view && !isCollapsed) {
            this._setCollapsed(true);
        } else {
            this.activeView = view;
            this._setCollapsed(false);
            this.updateVisibility();
            if (view === 'search' && options?.focus) {
                queueMicrotask(() => {
                    (this.views.get('search') as SearchToolView | undefined)?.focusInput();
                });
            }
        }
    }

    private _setCollapsed(collapsed: boolean): void {
        const main = document.getElementById('main_web_ui');
        if (!main) return;
        main.classList.toggle(LEFT_COLLAPSED_CLASS, collapsed);
    }

    private render(): void {
        this.rendered = true;
        this.innerHTML = '';
        this.classList.add('web-tool-host');

        const navTree = new NavTreeToolView().init({ itemApi: API.Item });
        const calendar = new CalendarToolView();
        const search = new SearchToolView().init({ itemApi: API.Item });
        const random = new RandomToolView().init({ itemApi: API.Item });
        this.views.set('nav_tree', navTree);
        this.views.set('calendar', calendar);
        this.views.set('search', search);
        this.views.set('random', random);

        this.views.forEach((view) => {
            this.appendChild(view);
        });

        this.updateVisibility();
    }

    private updateVisibility(): void {
        this.views.forEach((view, name) => {
            view.hidden = name !== this.activeView;
        });
    }

}

if (!customElements.get('web-tool-host')) {
    customElements.define('web-tool-host', WebToolHost);
}
