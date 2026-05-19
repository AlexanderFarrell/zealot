export interface ContextAction {
    label?: string;
    onClick?: () => void;
    danger?: boolean;
    separator?: boolean;
}

const registry = new Map<HTMLElement, () => ContextAction[]>();

export function registerContextMenu(element: HTMLElement, getActions: () => ContextAction[]): void {
    registry.set(element, getActions);
}

export function unregisterContextMenu(element: HTMLElement): void {
    registry.delete(element);
}

export function unregisterContextMenuIn(container: HTMLElement): void {
    for (const [el] of registry) {
        if (container.contains(el)) {
            registry.delete(el);
        }
    }
}

function isEditableTarget(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) return false;
    if (target.isContentEditable) return true;
    return target.closest('input, textarea, select, [contenteditable]') !== null;
}

function findRegisteredAncestor(target: EventTarget | null): HTMLElement | null {
    if (!(target instanceof Node)) return null;
    let node: Node | null = target as Node;
    while (node) {
        if (node instanceof HTMLElement && registry.has(node)) {
            return node;
        }
        node = node.parentNode;
    }
    return null;
}

let overlay: HTMLDivElement | null = null;

function getOrCreateOverlay(): HTMLDivElement {
    if (!overlay) {
        overlay = document.createElement('div');
        overlay.id = 'context-menu-overlay';
        document.body.appendChild(overlay);
    }
    return overlay;
}

function closeMenu(): void {
    if (!overlay) return;
    overlay.style.display = 'none';
    overlay.innerHTML = '';
}

function showMenu(actions: ContextAction[], x: number, y: number): void {
    const el = getOrCreateOverlay();
    el.innerHTML = '';

    const itemEls: HTMLButtonElement[] = [];

    for (const action of actions) {
        if (action.separator) {
            const hr = document.createElement('hr');
            hr.className = 'ctx-menu-separator';
            el.appendChild(hr);
            continue;
        }
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'ctx-menu-item';
        if (action.danger) btn.classList.add('ctx-menu-item--danger');
        btn.textContent = action.label ?? '';
        btn.addEventListener('click', (e) => {
            e.stopPropagation();
            closeMenu();
            action.onClick?.();
        });
        el.appendChild(btn);
        itemEls.push(btn);
    }

    // Position off-screen first to measure
    el.style.left = '-9999px';
    el.style.top = '-9999px';
    el.style.display = 'block';

    const rect = el.getBoundingClientRect();
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    el.style.left = `${Math.min(x, vw - rect.width - 8)}px`;
    el.style.top = `${Math.min(y, vh - rect.height - 8)}px`;

    const handleKeydown = (e: KeyboardEvent): void => {
        if (e.key === 'Escape') {
            closeMenu();
            return;
        }
        if (e.key === 'ArrowDown') {
            e.preventDefault();
            const focused = document.activeElement;
            const idx = itemEls.indexOf(focused as HTMLButtonElement);
            itemEls[(idx + 1) % itemEls.length]?.focus();
        }
        if (e.key === 'ArrowUp') {
            e.preventDefault();
            const focused = document.activeElement;
            const idx = itemEls.indexOf(focused as HTMLButtonElement);
            itemEls[(idx - 1 + itemEls.length) % itemEls.length]?.focus();
        }
    };

    el.addEventListener('keydown', handleKeydown);
}

if (typeof document !== 'undefined') {
    document.addEventListener('contextmenu', (e: MouseEvent) => {
        if (isEditableTarget(e.target)) return;
        const el = findRegisteredAncestor(e.target);
        if (!el) return;
        e.preventDefault();
        const getActions = registry.get(el);
        if (!getActions) return;
        const actions = getActions();
        if (actions.length === 0) return;
        showMenu(actions, e.clientX, e.clientY);
    });

    document.addEventListener('click', () => {
        closeMenu();
    }, { capture: true });

    document.addEventListener('keydown', (e: KeyboardEvent) => {
        if (e.key === 'Escape') closeMenu();
    });

    window.addEventListener('scroll', () => {
        closeMenu();
    }, { passive: true });
}
