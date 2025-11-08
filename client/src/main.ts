import './assets/style.css'
import './components/side_buttons.ts';
import './components/titlebar.ts';
import './components/content.ts';
import commands from './core/command_runner.ts';
import { events } from './core/events.ts';
import { CTRL_OR_META_KEY, Hotkey, register_hotkey, SHIFT_KEY } from './core/hotkeys.ts';

document.querySelector<HTMLDivElement>('#app')!.innerHTML = 
`<title-bar></title-bar>
<side-buttons></side-buttons>
<content></content>`

commands.register('Go to Home Page',
    [new Hotkey('h', [CTRL_OR_META_KEY])],
    () => {events.emit('Home')});
commands.register('Search Items', 
    [new Hotkey('o', [CTRL_OR_META_KEY])],
    () => {});
commands.register('Run Command', 
    [new Hotkey('p', [CTRL_OR_META_KEY])],
    () => {});
commands.register('Open Nav Sidebar', 
    [new Hotkey('t', [CTRL_OR_META_KEY])],
    () => {events.emit('Nav')});
commands.register('Open Media', 
    [new Hotkey('1', [CTRL_OR_META_KEY, SHIFT_KEY])],
    () => {events.emit('Media')});
commands.register('Up to Parent', () => {events.emit('Parent')});
commands.register('Open Calendar', () => {events.emit('Calendar')});
commands.register('Open Daily Planner', () => {events.emit('Daily')});
commands.register('Open Weekly Planner', () => {events.emit('Weekly')});
commands.register('Open Monthly Planner', () => {events.emit('Monthly')});
commands.register('Open Annual Planner', () => {events.emit('Annual')});
commands.register('Open Analysis', () => {events.emit('Analysis')});
commands.register('Open Rules Editor', () => {events.emit('Rules')});
commands.register('Open Settings', () => {events.emit('Settings')});
commands.register('New Item', () => {});

register_hotkey(new Hotkey('h', [CTRL_OR_META_KEY], () => {commands.run('Go to Home Page')}));
register_hotkey(new Hotkey())