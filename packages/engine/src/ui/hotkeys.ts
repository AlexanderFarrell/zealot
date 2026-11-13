export const CTRL_OR_META_KEY = 1;
export const ALT_KEY = 2;
export const SHIFT_KEY = 3;

export class Hotkey {
    public key: string;
    public shift_key: boolean;
    public alt_key: boolean;
    public ctrl_or_meta_key: boolean;
    public func: () => void;
    // Keys that never type a character (such as zoom) may fire while the
    // cursor is in an editor or text field.
    public allow_in_editable: boolean;

    constructor(key: string, helper_keys: number[], func: () => void = () => {}, allow_in_editable = false) {
        this.key = key.toLowerCase();
        this.shift_key = helper_keys.includes(SHIFT_KEY);
        this.alt_key = helper_keys.includes(ALT_KEY);
        this.ctrl_or_meta_key = helper_keys.includes(CTRL_OR_META_KEY);
        this.func = func;
        this.allow_in_editable = allow_in_editable;
    }

    toString() {
        let s = "";
        if (this.ctrl_or_meta_key) {
            s += "CTRL+"
        }
        if (this.alt_key) {
            s += "ALT+"
        }
        if (this.shift_key) {
            s += "SHIFT+"
        }
        s += this.key;
        return s.toUpperCase();
    }
}

const keys = new Map<string, Hotkey>();

function getHotkeySignature(key: string, shiftKey: boolean, altKey: boolean, ctrlOrMetaKey: boolean): string {
    return `${ctrlOrMetaKey ? 1 : 0}:${altKey ? 1 : 0}:${shiftKey ? 1 : 0}:${key}`;
}

function xor(a: boolean, b: boolean):boolean {
    return (a || b) && (!a && b);
}

export function normalizeEventKey(e: KeyboardEvent): string {
    // Zoom keys match by physical key so `=` and `+` (Shift+`=`) are one key,
    // and the numpad `+` and `-` count too.
    if (e.code === 'Equal' || e.code === 'NumpadAdd') return '=';
    if (e.code === 'Minus' || e.code === 'NumpadSubtract') return '-';
    if (e.code.startsWith('Key')) {
        return e.code.slice(3).toLowerCase();
    }
    if (e.code.startsWith('Digit')) {
        return e.code.slice(5);
    }
    return e.key.toLowerCase();
}

function isEditableTarget(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) {
        return false;
    }
    if (target.isContentEditable) {
        return true;
    }
    return target.closest('input, textarea, select, [contenteditable]') !== null;
}

export function register_hotkey(key: Hotkey) {
    keys.set(
        getHotkeySignature(key.key, key.shift_key, key.alt_key, key.ctrl_or_meta_key),
        key,
    );
}

export function clear_hotkeys(): void {
    keys.clear();
}

const isMac = (): boolean => typeof navigator !== 'undefined' && navigator.platform.toUpperCase().includes('MAC');

export function handle_keydown(e: KeyboardEvent): void {
    const modifier = isMac() ? e.metaKey : e.ctrlKey;
    const hotkey = keys.get(getHotkeySignature(normalizeEventKey(e), e.shiftKey, e.altKey, modifier));

    if (!hotkey) {
        return;
    }
    if (isEditableTarget(e.target) && !hotkey.allow_in_editable) {
        return;
    }

    if ((!xor(e.shiftKey, hotkey.shift_key)) &&
        (!xor(e.altKey, hotkey.alt_key)) &&
        (!xor(modifier, hotkey.ctrl_or_meta_key)))
    {
        hotkey.func();
        e.preventDefault();
    }
}

// Hotkeys listen on the main document; popout windows get the same listener.
export function attach_hotkeys(doc: Document): void {
    doc.addEventListener('keydown', handle_keydown);
}

if (typeof document !== 'undefined') {
    attach_hotkeys(document);
}
