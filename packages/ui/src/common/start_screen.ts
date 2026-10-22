import { AppSettings } from '@websoil/engine';
import { ItemAPI } from '@zealot/api/src/item';

const LAST_ITEM_KEY = 'zealot:last-item';

const itemApi = new ItemAPI('/api');

/** Remembers the item the user is looking at, per device, for the "Last item" start screen. */
export function rememberLastItem(itemId: number): void {
    try {
        localStorage.setItem(LAST_ITEM_KEY, String(itemId));
    } catch {
        // Storage can be unavailable (private windows); the start screen then falls back to Home.
    }
}

function readLastItem(): number | null {
    try {
        const id = Number(localStorage.getItem(LAST_ITEM_KEY));
        return Number.isInteger(id) && id > 0 ? id : null;
    } catch {
        return null;
    }
}

/** Today as YYYY-MM-DD in the browser's time zone. */
function today(): string {
    return new Date().toLocaleDateString('en-CA');
}

/**
 * Opens the user's chosen start screen. Only the bare root URL at launch counts: a deep link, or
 * moving around the app, is never redirected. Returns true when it redirected straight away, so
 * the caller can skip drawing Home first. "Last item" has to check the item still exists, so it
 * redirects a moment later and Home stays up if the item is gone.
 */
export function openStartScreen(path: string, navigate: (path: string) => void): boolean {
    if (path !== '/') return false;
    const choice = AppSettings.get().start_screen;
    if (choice === 'daily') {
        navigate(`/planner/daily/${today()}`);
        return true;
    }
    if (choice === 'last_item') {
        const id = readLastItem();
        if (id != null) {
            itemApi.GetById(id).then(() => navigate(`/item_id/${id}`)).catch(() => {
                // The item is gone or unreachable; stay on Home.
            });
        }
    }
    return false;
}
