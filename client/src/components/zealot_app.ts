import AuthAPI from '../api/auth.ts';
import { events } from '../core/events';
import { CTRL_OR_META_KEY, Hotkey, SHIFT_KEY } from '../core/hotkeys';
import { router, setup_router } from '../core/router.ts';
import commands from './../core/command_runner.ts';
import AddItemModal from './add_item_modal';

class ZealotApp extends HTMLElement {
    connectedCallback() {
        this.id = "app";
        this.innerHTML = `<title-bar></title-bar>
<div id="main-display">
<side-buttons></side-buttons>
<side-bar id="left-side-bar"><nav-view></nav-view></side-bar>
<content-></content->
<side-bar id="right-side-bar"><item-attributes-view></item-attributes-view></side-bar>
</div>`;
        this.setup_commands();
        setup_router();

        let link = window.location.pathname + window.location.search + window.location.hash;
        console.log(link)
        router.navigate('/analysis')
        router.navigate(link)
    }

    disconnectedCallback() {
        commands.clear();
    }

    setup_commands() {
        commands.register('Go to Home Page',
            [new Hotkey('h', [CTRL_OR_META_KEY])],
            () => {
                router.navigate('/');
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
                router.navigate('/media')
            });
        commands.register('Up to Parent', 
            [new Hotkey('UpArrow', [CTRL_OR_META_KEY])],
            () => {
                events.emit('Parent')
                console.error("Not implemented Up to Parent command")
            });
        commands.register('Open Calendar', 
            [new Hotkey('r', [CTRL_OR_META_KEY])],
            () => {
                let left_sidebar = document.querySelector('#left-side-bar')!;
                left_sidebar.innerHTML = "<calendar-view></calendar-view>"
            });
        commands.register('Open Daily Planner', 
            [new Hotkey('1', [CTRL_OR_META_KEY])],
            () => {
                router.navigate('/planner/daily');
            });
        commands.register('Open Weekly Planner', 
            [new Hotkey('2', [CTRL_OR_META_KEY])],
            () => {
                router.navigate('/planner/weekly')
            });
        commands.register('Open Monthly Planner', 
            [new Hotkey('3', [CTRL_OR_META_KEY])],
            () => {
                router.navigate('/planner/monthly')
            });
        commands.register('Open Annual Planner', 
            [new Hotkey('4', [CTRL_OR_META_KEY])],
            () => {
                router.navigate('/planner/annual')
            });
        commands.register('Open Analysis', 
            [new Hotkey('1', [CTRL_OR_META_KEY, SHIFT_KEY])],
            () => {
                router.navigate('/analysis')
            });
        commands.register('Open Rules Editor', 
            [new Hotkey('2', [CTRL_OR_META_KEY, SHIFT_KEY])],
            () => {
                router.navigate('/rules')
            });
        commands.register('Open Settings', 
            [new Hotkey('3', [CTRL_OR_META_KEY, SHIFT_KEY])],
            () => {
                router.navigate('/settings')
            });
        commands.register('New Item', 
            [new Hotkey('n', [CTRL_OR_META_KEY])],
            () => {
                document.body.appendChild(new AddItemModal());
            });
        commands.register('Logout',
            [],
            async () => {
                await AuthAPI.logout();
                events.emit('on_logout');
            }
        )
    }
}

customElements.define('zealot-app', ZealotApp)

export default ZealotApp;