import { MobilePaneState, type RightSidebarHost, type SideViewMeta } from '@websoil/engine';

export class RightSidebar extends HTMLElement implements RightSidebarHost {
    setContent(el: HTMLElement | null, meta?: SideViewMeta): void {
        this.innerHTML = '';
        if (el) {
            this.appendChild(el);
        }
        MobilePaneState.setSide(el ? { label: meta?.label ?? 'Side', count: meta?.count ?? null } : null);
    }

    setCount(count: number | null): void {
        MobilePaneState.setSideCount(count);
    }
}

customElements.define('right-sidebar', RightSidebar);
