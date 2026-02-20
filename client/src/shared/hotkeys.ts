export const CTRL_OR_META_KEY = 1;
export const ALT_KEY = 2;
export const SHIFT_KEY = 3;

export class Hotkey {
    public key: string;
    public shift_key: boolean;
    public alt_key: boolean;
    public ctrl_or_meta_key: boolean; 
    public func: Function;

    constructor(key: string, helper_keys: Number[], func: Function = () => {}) {
        this.key = key;
        this.shift_key = helper_keys.includes(SHIFT_KEY);
        this.alt_key = helper_keys.includes(ALT_KEY);
        this.ctrl_or_meta_key = helper_keys.includes(CTRL_OR_META_KEY);
        this.func = func;
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

function xor(a: boolean, b: boolean):boolean {
    return (a || b) && (!a && b);
}

export function register_hotkey(key: Hotkey) {
    keys.set(key.key, key);
}

document.body.addEventListener('keydown', (e: KeyboardEvent) => {
    if (keys.has(e.key)) {
        let hotkey = keys.get(e.key)!;
        let isMac = navigator.platform.toUpperCase().includes('MAC');

        let modifier = isMac ? e.metaKey : e.ctrlKey;
        if ((!xor(e.shiftKey, hotkey.shift_key)) &&
            (!xor(e.altKey, hotkey.alt_key)) &&
            (!xor(modifier, hotkey.ctrl_or_meta_key))) 
        {
            hotkey.func();
            e.preventDefault();
        }
    }
})