import { CTRL_OR_META_KEY, Hotkey, SHIFT_KEY } from "./hotkeys";
import runner from "./commands";

// Chrome's zoom steps, in percent.
export const ZOOM_STEPS = [25, 33, 50, 67, 75, 80, 90, 100, 110, 125, 150, 175, 200, 250, 300, 400, 500] as const;
export const ZOOM_DEFAULT = 100;
const ZOOM_MIN = 25;
const ZOOM_MAX = 500;
export const ZOOM_KEY = "zealot:zoom";

export const ZoomCommands = {
    zoomIn: "Zoom In",
    zoomOut: "Zoom Out",
    resetZoom: "Reset Zoom",
} as const;

/** Nearest step; a missing or invalid value is 100%. */
export function snapZoom(value: unknown): number {
    const n = typeof value === "string" ? Number(value) : value;
    if (typeof n !== "number" || !Number.isFinite(n) || n <= 0) return ZOOM_DEFAULT;
    let best: number = ZOOM_DEFAULT;
    for (const step of ZOOM_STEPS) {
        if (Math.abs(step - n) < Math.abs(best - n)) best = step;
    }
    return best;
}

/** One step up or down from `level`, stopping at the ends. */
export function stepZoom(level: number, direction: 1 | -1): number {
    const i = ZOOM_STEPS.indexOf(snapZoom(level) as typeof ZOOM_STEPS[number]);
    const next = Math.min(ZOOM_STEPS.length - 1, Math.max(0, i + direction));
    return ZOOM_STEPS[next] ?? ZOOM_DEFAULT;
}

export function loadZoom(): number {
    try {
        return snapZoom(localStorage.getItem(ZOOM_KEY));
    } catch {
        return ZOOM_DEFAULT;
    }
}

function saveZoom(level: number): void {
    try {
        localStorage.setItem(ZOOM_KEY, String(level));
    } catch {
        // Storage unavailable: the level just won't persist.
    }
}

type Applier = (level: number) => void | Promise<void>;
let level = ZOOM_DEFAULT;
let applier: Applier | null = null;
const listeners = new Set<(level: number) => void>();

export function getZoom(): number {
    return level;
}

export function canZoomIn(): boolean {
    return level < ZOOM_MAX;
}

export function canZoomOut(): boolean {
    return level > ZOOM_MIN;
}

export function onZoomChange(cb: (level: number) => void): () => void {
    listeners.add(cb);
    return () => listeners.delete(cb);
}

export function setZoom(next: number): void {
    level = snapZoom(next);
    saveZoom(level);
    void applier?.(level);
    listeners.forEach((cb) => cb(level));
}

export const zoomIn = (): void => setZoom(stepZoom(level, 1));
export const zoomOut = (): void => setZoom(stepZoom(level, -1));
export const resetZoom = (): void => setZoom(ZOOM_DEFAULT);

/** Desktop calls this with the function that really scales the window, then the stored level is applied. */
export function initZoom(apply: Applier): void {
    applier = apply;
    level = loadZoom();
    void apply(level);
    listeners.forEach((cb) => cb(level));
}

/** Zoom keys also work while typing, so they opt in to firing in editable targets. */
export function registerZoomCommands(): void {
    const ctrl = [CTRL_OR_META_KEY];
    const ctrlShift = [CTRL_OR_META_KEY, SHIFT_KEY];
    const hk = (key: string, mods: number[]) => new Hotkey(key, mods, () => {}, true);
    runner.register(ZoomCommands.zoomIn, [hk("=", ctrl), hk("=", ctrlShift)], zoomIn);
    runner.register(ZoomCommands.zoomOut, [hk("-", ctrl)], zoomOut);
    runner.register(ZoomCommands.resetZoom, [hk("0", ctrl)], resetZoom);
}
