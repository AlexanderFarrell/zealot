import AuthAPI from '../api/auth.ts';
import ItemAPI from '../api/item.ts';
import { events } from '../core/events';
import { CTRL_OR_META_KEY, Hotkey, SHIFT_KEY } from '../core/hotkeys';
import { switch_item_to } from '../screens/item_screen';
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
    }

    disconnectedCallback() {
        commands.clear();
    }

    setup_commands() {
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


events.on('on_register_account', () => {
    ItemAPI.add('Home')
})

export default ZealotApp;