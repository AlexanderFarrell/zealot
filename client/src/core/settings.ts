let settings: any | null = null;
let validator = (s: any) => {return s};

export function set_settings(s: any) {
    settings = validator(s);
}

export function get_settings() {
    return settings;
}

export function set_settings_validator(v: (s: any) => any) {
    validator = v;
}
