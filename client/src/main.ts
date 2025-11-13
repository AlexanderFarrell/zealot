import './assets/style.css'
import './components/side_buttons.ts';
import './components/titlebar.ts';
import './components/content.ts';
import './components/sidebar.ts';
import './components/sidebars/nav_view.ts';
import './components/sidebars/search_view.ts';
import './screens/item_screen.ts';
import commands from './core/command_runner.ts';
import { events } from './core/events.ts';
import { ALT_KEY, CTRL_OR_META_KEY, Hotkey, register_hotkey, SHIFT_KEY } from './core/hotkeys.ts';
import {settings} from './core/settings.ts';
import AddItemModal from './components/add_item_modal.ts';

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
    () => {events.emit('Home')});
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
    () => {events.emit('Media')});
commands.register('Up to Parent', 
    [new Hotkey('UpArrow', [CTRL_OR_META_KEY])],
    () => {events.emit('Parent')});
commands.register('Open Calendar', 
    [new Hotkey('r', [CTRL_OR_META_KEY])],
    () => {events.emit('Calendar')});
commands.register('Open Daily Planner', 
    [new Hotkey('1', [CTRL_OR_META_KEY])],
    () => {events.emit('Daily')});
commands.register('Open Weekly Planner', 
    [new Hotkey('2', [CTRL_OR_META_KEY])],
    () => {events.emit('Weekly')});
commands.register('Open Monthly Planner', 
    [new Hotkey('3', [CTRL_OR_META_KEY])],
    () => {events.emit('Monthly')});
commands.register('Open Annual Planner', 
    [new Hotkey('4', [CTRL_OR_META_KEY])],
    () => {events.emit('Annual')});
commands.register('Open Analysis', 
    [new Hotkey('1', [CTRL_OR_META_KEY, SHIFT_KEY])],
    () => {events.emit('Analysis')});
commands.register('Open Rules Editor', 
    [new Hotkey('2', [CTRL_OR_META_KEY, SHIFT_KEY])],
    () => {events.emit('Rules')});
commands.register('Open Settings', 
    [new Hotkey('3', [CTRL_OR_META_KEY, SHIFT_KEY])],
    () => {events.emit('Settings')});
commands.register('New Item', 
    [new Hotkey('n', [CTRL_OR_META_KEY])],
    () => {
        document.body.appendChild(new AddItemModal());
    });
