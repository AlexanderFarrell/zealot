import './assets/style.css'
import './components/side_buttons.ts';
import './components/titlebar.ts';
import './components/content.ts';
import './components/sidebar.ts';
import './components/sidebars/nav_view.ts';
import './components/sidebars/search_view.ts';
import './components/sidebars/calendar_view.ts';
import './screens/item_screen.ts';

import './screens/analysis_screen.ts';
import './screens/day_screen.ts';
import './screens/media_screen.ts';
import './screens/month_screen.ts';
import './screens/rules_screen.ts';
import './screens/settings_screen.ts';
import './screens/week_screen.ts';
import './screens/year_screen.ts';
import { events } from './core/events.ts';
import AuthModal from './components/auth_modal.ts';
import AuthAPI from './api/auth.ts';
import ZealotApp from './components/zealot_app.ts';
import { set_settings_validator } from './core/settings.ts';


document.querySelector<HTMLDivElement>('#app')!.innerHTML = 
`Loading Zealot...`


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