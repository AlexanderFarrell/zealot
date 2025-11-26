import './assets/style.css'
import './components/side_buttons.ts';
import './components/titlebar.ts';
import './components/content.ts';
import './components/sidebar.ts';
import './components/sidebars/nav_view.ts';
import './components/sidebars/search_view.ts';
import './screens/item_screen.ts';

import './screens/analysis_screen.ts';
import './screens/day_screen.ts';
import './screens/media_screen.ts';
import './screens/month_screen.ts';
import './screens/rules_screen.ts';
import './screens/settings_screen.ts';
import './screens/week_screen.ts';
import './screens/year_screen.ts';
import commands from './core/command_runner.ts';
import { events } from './core/events.ts';
import { ALT_KEY, CTRL_OR_META_KEY, Hotkey, register_hotkey, SHIFT_KEY } from './core/hotkeys.ts';
import {settings} from './core/settings.ts';
import AddItemModal from './components/add_item_modal.ts';
import { switch_item_to } from './screens/item_screen.ts';
import AuthModal from './components/auth_modal.ts';
import AuthAPI from './api/auth.ts';
import ZealotApp from './components/zealot_app.ts';

settings.set('host', '127.0.0.1:8082');

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
        document.body.appendChild(new ZealotApp());
    }
})()