export interface Settings {
    theme: 'light' | 'dark';
    show_tips: boolean;
}

const DEFAULTS: Settings = {
    theme: 'light',
    show_tips: true,
};

const STORAGE_KEY = 'zealot_settings';

type Listener = (settings: Settings) => void;

let current: Settings = load();
const listeners = new Set<Listener>();
let syncTimer: ReturnType<typeof setTimeout> | null = null;
let syncFn: ((settings: Settings) => void) | null = null;

function load(): Settings {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (raw) {
            return { ...DEFAULTS, ...JSON.parse(raw) };
        }
    } catch {
        // ignore parse errors
    }
    return { ...DEFAULTS };
}

function save(settings: Settings): void {
    try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
    } catch {
        // ignore storage errors (e.g. private browsing quota)
    }
}

function notify(): void {
    for (const listener of listeners) {
        listener(current);
    }
}

function scheduleSync(): void {
    if (syncFn === null) return;
    if (syncTimer !== null) clearTimeout(syncTimer);
    syncTimer = setTimeout(() => {
        syncTimer = null;
        syncFn!(current);
    }, 500);
}

/** Read current settings. */
function get(): Settings {
    return current;
}

/** Update one or more settings keys. Persists to localStorage and schedules a debounced server sync. */
function set(partial: Partial<Settings>): void {
    current = { ...current, ...partial };
    save(current);
    notify();
    scheduleSync();
}

/**
 * Called after login. Merges server settings (server wins), writes back to localStorage.
 * Does not trigger a sync back to the server.
 */
function loadFromServer(raw: Record<string, unknown>): void {
    current = { ...current, ...raw } as Settings;
    save(current);
    notify();
}

/** Subscribe to settings changes. Returns an unsubscribe function. */
function subscribe(listener: Listener): () => void {
    listeners.add(listener);
    return () => listeners.delete(listener);
}

/**
 * Register the function used to sync settings to the server.
 * Call this once during app initialisation, passing in the API method.
 * Example: Settings.registerSync((s) => api.auth.patchSettings(s));
 */
function registerSync(fn: (settings: Settings) => void): void {
    syncFn = fn;
}

export const AppSettings = { get, set, loadFromServer, subscribe, registerSync };
