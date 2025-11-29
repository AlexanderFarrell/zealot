export let settings: any | null = null;
let validator = (s: any) => {return s};

export function set_settings(s: any) {
    settings = s;
    validator(settings);
}

export function set_settings_validator(v: (s: any) => any) {
    validator = v;
}

export default settings;