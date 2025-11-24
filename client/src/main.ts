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

settings.set('host', '127.0.0.1:8082');

document.querySelector<HTMLDivElement>('#app')!.innerHTML = 
`<title-bar></title-bar>
<div id="main-display">
<side-buttons></side-buttons>
<side-bar id="left-side-bar"><nav-view></nav-view></side-bar>
<content-></content->
<side-bar id="right-side-bar"></side-bar>
</div>`

commands.register('Go to Home Page',
    [new Hotkey('h', [CTRL_OR_META_KEY])],
    () => {
        switch_item_to('Home');
    });
commands.register('Search Items', 
    [new Hotkey('k', [CTRL_OR_META_KEY, SHIFT_KEY])],
    () => {
        let left_sidebar = document.querySelector('#left-side-bar')!;
        left_sidebar.innerHTML = "<search-view></search-view>";
    });
commands.register('Run Command', 
    [new Hotkey('p', [CTRL_OR_META_KEY])],
    () => {});
commands.register('Open Nav Sidebar', 
    [new Hotkey('t', [CTRL_OR_META_KEY])],
    () => {
        let left_sidebar = document.querySelector("#left-side-bar")!;
        left_sidebar.innerHTML = "<nav-view></nav-view>";
    });
commands.register('Open Media', 
    [new Hotkey('m', [CTRL_OR_META_KEY])],
    () => {
        document.querySelector('content-')!.innerHTML = "<media-screen></media-screen>";
    });
commands.register('Up to Parent', 
    [new Hotkey('UpArrow', [CTRL_OR_META_KEY])],
    () => {events.emit('Parent')});
commands.register('Open Calendar', 
    [new Hotkey('r', [CTRL_OR_META_KEY])],
    () => {events.emit('Calendar')});
commands.register('Open Daily Planner', 
    [new Hotkey('1', [CTRL_OR_META_KEY])],
    () => {
        document.querySelector('content-')!.innerHTML = "<daily-planner-screen></daily-planner-screen>";
    });
commands.register('Open Weekly Planner', 
    [new Hotkey('2', [CTRL_OR_META_KEY])],
    () => {
        document.querySelector('content-')!.innerHTML = "<weekly-planner-screen></weekly-planner-screen>";
    });
commands.register('Open Monthly Planner', 
    [new Hotkey('3', [CTRL_OR_META_KEY])],
    () => {
        document.querySelector('content-')!.innerHTML = "<monthly-planner-screen></monthly-planner-screen>";
    });
commands.register('Open Annual Planner', 
    [new Hotkey('4', [CTRL_OR_META_KEY])],
    () => {
        document.querySelector('content-')!.innerHTML = "<annual-planner-screen></annual-planner-screen>";
    });
commands.register('Open Analysis', 
    [new Hotkey('1', [CTRL_OR_META_KEY, SHIFT_KEY])],
    () => {
        document.querySelector('content-')!.innerHTML = "<analysis-screen></analysis-screen>";
    });
commands.register('Open Rules Editor', 
    [new Hotkey('2', [CTRL_OR_META_KEY, SHIFT_KEY])],
    () => {
        document.querySelector('content-')!.innerHTML = "<rules-screen></rules-screen>";
    });
commands.register('Open Settings', 
    [new Hotkey('3', [CTRL_OR_META_KEY, SHIFT_KEY])],
    () => {
        document.querySelector('content-')!.innerHTML = "<settings-screen></settings-screen>";
    });
commands.register('New Item', 
    [new Hotkey('n', [CTRL_OR_META_KEY])],
    () => {
        document.body.appendChild(new AddItemModal());
    });

events.on('on_logout', () => {
    document.body.appendChild(new AuthModal());
});

(async () => {
    if (!await AuthAPI.is_logged_in()) {
        document.body.appendChild(new AuthModal());
    }
})()