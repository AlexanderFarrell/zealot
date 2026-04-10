import type { ToolHost, ToolShowOptions, ToolView } from '@websoil/engine';
import { CalendarToolView, NavTreeToolView, SearchToolView } from '@zealot/ui';
import { API } from './core';

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
        this.updateVisibility();
        if (view === 'search' && options?.focus) {
            queueMicrotask(() => {
                (this.views.get('search') as SearchToolView | undefined)?.focusInput();
            });
        }
    }

    private render(): void {
        this.rendered = true;
        this.innerHTML = '';
        this.classList.add('web-tool-host');

        const navTree = new NavTreeToolView().init({ itemApi: API.Item });
        const calendar = new CalendarToolView();
        const search = new SearchToolView().init({ itemApi: API.Item });
        const attributesPlaceholder = this.buildPlaceholder();

        this.views.set('nav_tree', navTree);
        this.views.set('calendar', calendar);
        this.views.set('search', search);
        this.views.set('item_attributes', attributesPlaceholder);

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

    private buildPlaceholder(): HTMLElement {
        const placeholder = document.createElement('div');
        placeholder.className = 'tool-panel tool-placeholder';
        placeholder.innerHTML = `
        <div class="tool-panel-header">
            <h2>Item Attributes</h2>
        </div>
        <p class="tool-muted">This tool is reserved for TASK-008 and is not implemented in the web client yet.</p>
        `;
        return placeholder;
    }
}

if (!customElements.get('web-tool-host')) {
    customElements.define('web-tool-host', WebToolHost);
}
