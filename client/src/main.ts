import './assets/css/main.scss';
import "highlight.js/styles/github-dark.css";
import "./element_map.ts";

import AuthModal from './features/auth/auth_modal.ts';
import AuthAPI from './api/auth.ts';
import ZealotApp from './app/zealot_app.ts';
import { events } from './shared/events.ts';
import { set_settings_validator } from './shared/settings.ts';


document.querySelector<HTMLDivElement>('#app')!.innerHTML = `Loading Zealot...`

events.on('on_logout', () => {
    document.body.innerHTML = "";
    document.body.appendChild(new AuthModal());
});

(async () => {
    if (!await AuthAPI.is_logged_in()) {
        document.body.innerHTML = "";
        document.body.appendChild(new AuthModal());
    } else {
        document.body.innerHTML = "";
        try {
            await AuthAPI.get_user_details();
        } catch (e) {
            console.error(e)
        }
        document.body.appendChild(new ZealotApp());
    }
})()

set_settings_validator((s: any) => {
    const DEFAULT_SETTINGS = {
        theme: "light",
        show_tips: true,
        attribute_metas: [],
        item_types: [],
    };

    return { ...DEFAULT_SETTINGS, ...s};
})
