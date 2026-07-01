import { DockviewComponent } from 'dockview-core';
import type { IContentRenderer, GroupPanelPartInitParameters } from 'dockview-core';
import { getRightSidebarHost } from '@websoil/engine';
import type { DesktopNavigator } from './desktop_navigator';

interface PanelParams {
    path: string;
}

// Map from panel id → content element, so getActiveContent() can find the div
// without going through updateParameters (which triggers update() → infinite loop risk).
const panelContentMap = new Map<string, HTMLElement>();

export class ZealotPanelContent implements IContentRenderer {
    readonly element: HTMLElement;
    private _api: GroupPanelPartInitParameters['api'] | null = null;
    private _dockHost: DesktopDockHost | null = null;

    constructor() {
        this.element = document.createElement('div');
        this.element.style.cssText = 'width:100%;height:100%;overflow:auto;padding:10px;box-sizing:border-box;';
    }

    setDockHost(host: DesktopDockHost): void {
        this._dockHost = host;
    }

    init(parameters: GroupPanelPartInitParameters): void {
        this._api = parameters.api;
        panelContentMap.set(parameters.api.id, this.element);
        const path = (parameters.params as PanelParams).path ?? '/';
        this._dockHost?.renderPath(this.element, path, parameters.api);
    }

    update(params: { params: Record<string, unknown> }): void {
        const path = (params.params as Partial<PanelParams>).path;
        if (path && this._api && this._dockHost) {
            this._dockHost.renderPath(this.element, path, this._api);
        }
    }

    dispose(): void {
        if (this._api) panelContentMap.delete(this._api.id);
        this.element.innerHTML = '';
    }
}

export class DesktopDockHost extends HTMLElement {
    private _dockview: DockviewComponent | null = null;
    private _navigator: DesktopNavigator | null = null;

    // Called from desktop_client.ts after the element is in the DOM.
    // Must be called before any tabs are opened.
    init(nav: DesktopNavigator): this {
        this._navigator = nav;

        this.style.cssText = 'display:block;width:100%;height:100vh;min-width:0;position:relative;';
        this.classList.add('dockview-theme-dark');

        this._dockview = new DockviewComponent(this, {
            createComponent: () => {
                const panel = new ZealotPanelContent();
                panel.setDockHost(this);
                return panel;
            },
            popoutUrl: '/popout.html',
        });

        this._dockview.api.onDidOpenPopoutWindowFail(() => {
            console.error('dockview: popup window was blocked — ensure core:window:allow-create capability is set');
        });

        // Detect when a tab is dragged outside all drop targets and open it as an OS window.
        this._dockview.api.onWillDragPanel((event) => {
            const panel = event.panel;
            event.nativeEvent.target?.addEventListener('dragend', (e) => {
                const de = e as DragEvent;
                if (de.dataTransfer?.dropEffect === 'none' && this._dockview) {
                    void this._dockview.api.addPopoutGroup(panel, {
                        position: { left: de.screenX - 40, top: de.screenY - 20, width: 900, height: 700 },
                    });
                }
            }, { once: true });
        });

        const refreshActivePanel = (): void => {
            const active = this._dockview?.api.activePanel;
            if (active) {
                const path = (active.params as Partial<PanelParams>).path ?? '/';
                history.replaceState(null, '', path);
                // Re-fire sidebar for the newly active tab's screen.
                const el = panelContentMap.get(active.id);
                const screen = el?.firstElementChild as (HTMLElement & { onActivated?(): void }) | null;
                if (screen?.onActivated) {
                    screen.onActivated();
                } else {
                    getRightSidebarHost()?.setContent(null);
                }
            }
        };

        this._dockview.api.onDidActivePanelChange(refreshActivePanel);
        // Dragging a tab to split creates a new group that becomes active without
        // necessarily changing which panel is "active" (same panel, new group), so
        // onDidActivePanelChange alone can miss it — also refresh on group change.
        this._dockview.api.onDidActiveGroupChange(refreshActivePanel);

        // When the last panel is closed, open a new Home tab automatically.
        this._dockview.api.onDidRemovePanel(() => {
            if (this._dockview && this._dockview.api.panels.length === 0) {
                this.openInNewTab('/');
            }
        });

        this.openInNewTab('/');
        return this;
    }

    connectedCallback(): void {
        // Intentionally empty — init() handles setup once the navigator is ready.
    }

    getActiveContent(): HTMLElement | null {
        const active = this._dockview?.api.activePanel;
        if (!active) return null;
        return panelContentMap.get(active.id) ?? null;
    }

    getActivePanelApi(): GroupPanelPartInitParameters['api'] | null {
        return this._dockview?.api.activePanel?.api ?? null;
    }

    openInCurrentTab(path: string): void {
        const active = this._dockview?.api.activePanel;
        if (!active) {
            this.openInNewTab(path);
            return;
        }
        const el = panelContentMap.get(active.id);
        if (el) this.renderPath(el, path, active.api);
    }

    openInNewTab(path: string): void {
        if (!this._dockview) return;
        const id = crypto.randomUUID();
        this._dockview.api.addPanel({
            id,
            component: 'zealot',
            title: '…',
            params: { path } satisfies PanelParams,
        });
    }

    renderPath(el: HTMLElement, path: string, api: GroupPanelPartInitParameters['api']): void {
        if (!this._navigator) return;
        this._navigator.renderInto(el, path, (title) => {
            // Defer past dockview's own setTitle(params.title) call which runs after view.init().
            queueMicrotask(() => api.setTitle(title));
        });
    }

    closeActiveTab(): void {
        this._dockview?.api.activePanel?.api.close();
    }

    disconnectedCallback(): void {
        this._dockview?.dispose();
        this._dockview = null;
    }
}

let _dockHost: DesktopDockHost | null = null;
export function setDockHost(h: DesktopDockHost): void { _dockHost = h; }
export function getDockHost(): DesktopDockHost { return _dockHost!; }

customElements.define('desktop-dock-host', DesktopDockHost);
