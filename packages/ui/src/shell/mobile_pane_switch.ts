import { MobilePaneState } from '@websoil/engine';
import { icons } from '@zealot/content';

/**
 * Phone-only `Content | Related (8)` switch that lives in a screen's top button
 * row (item actions, planner header). Hidden by CSS above 768px and whenever
 * the screen has no side view (Z165).
 */
export class MobilePaneSwitch extends HTMLElement {
    private unsubscribe: (() => void) | null = null;

    connectedCallback(): void {
        this.unsubscribe = MobilePaneState.subscribe(() => this.paint());
        this.paint();
    }

    disconnectedCallback(): void {
        this.unsubscribe?.();
        this.unsubscribe = null;
    }

    private paint(): void {
        const side = MobilePaneState.getSide();
        this.hidden = !side;
        this.innerHTML = '';
        if (!side) return;

        const active = MobilePaneState.getPane();
        if (this.dataset.variant === 'icon') {
            // One icon-only toggle, sized like the planner's other header buttons.
            const button = document.createElement('button');
            button.type = 'button';
            button.className = 'planner-header-button';
            const label = side.count == null ? side.label : `${side.label} (${side.count})`;
            button.setAttribute('aria-label', label);
            button.title = label;
            button.setAttribute('aria-pressed', String(active === 'side'));
            const icon = document.createElement('img');
            icon.alt = '';
            icon.src = icons.repeat;
            button.appendChild(icon);
            button.addEventListener('click', () => {
                MobilePaneState.setPane(MobilePaneState.getPane() === 'side' ? 'content' : 'side');
            });
            this.appendChild(button);
            return;
        }
        const tab = (label: string, pane: 'content' | 'side'): HTMLButtonElement => {
            const button = document.createElement('button');
            button.type = 'button';
            button.textContent = label;
            button.setAttribute('role', 'tab');
            button.setAttribute('aria-selected', String(active === pane));
            button.addEventListener('click', () => {
                if (MobilePaneState.getPane() === pane) return;
                MobilePaneState.setPane(pane);
            });
            return button;
        };

        this.setAttribute('role', 'tablist');
        this.appendChild(tab('Content', 'content'));
        this.appendChild(tab(side.count == null ? side.label : `${side.label} (${side.count})`, 'side'));
    }
}

customElements.define('mobile-pane-switch', MobilePaneSwitch);

export function createMobilePaneSwitch(variant?: 'icon'): MobilePaneSwitch {
    const el = document.createElement('mobile-pane-switch') as MobilePaneSwitch;
    if (variant) el.dataset.variant = variant;
    return el;
}
