import type { RightSidebarHost } from '@websoil/engine';

export class RightSidebar extends HTMLElement implements RightSidebarHost {
    setContent(el: HTMLElement | null): void {
        this.innerHTML = '';
        if (el) {
            this.appendChild(el);
        }
    }
}

customElements.define('right-sidebar', RightSidebar);
