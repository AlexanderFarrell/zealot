/**
 * Phone layout shows either the main content or the side view (the right
 * sidebar: Related on an item, Repeats on the daily planner), one at a time.
 * The choice is global and remembered, so picking Related once keeps later
 * items on Related (Z165).
 */
export type MobilePane = 'content' | 'side';

export interface MobileSide {
    label: string;
    count: number | null;
}

const STORAGE_KEY = 'zealot_mobile_pane';

type Listener = () => void;

let pane: MobilePane = readStored();
let side: MobileSide | null = null;
const listeners = new Set<Listener>();

function readStored(): MobilePane {
    try {
        return localStorage.getItem(STORAGE_KEY) === 'side' ? 'side' : 'content';
    } catch {
        return 'content';
    }
}

function emit(): void {
    document.body?.setAttribute('data-mobile-pane', pane);
    if (side) document.body?.setAttribute('data-mobile-side', '');
    else document.body?.removeAttribute('data-mobile-side');
    listeners.forEach((fn) => fn());
}

export const MobilePaneState = {
    getPane(): MobilePane {
        return pane;
    },

    /** The side view when a screen has one, else null (strip hidden, content shown). */
    getSide(): MobileSide | null {
        return side;
    },

    setPane(next: MobilePane): void {
        pane = next;
        try {
            localStorage.setItem(STORAGE_KEY, next);
        } catch {
            // Storage unavailable (private window): the choice just won't outlive the page.
        }
        emit();
    },

    setSide(next: MobileSide | null): void {
        side = next;
        emit();
    },

    setSideCount(count: number | null): void {
        if (!side) return;
        side = { ...side, count };
        emit();
    },

    subscribe(fn: Listener): () => void {
        listeners.add(fn);
        return () => listeners.delete(fn);
    },
};
