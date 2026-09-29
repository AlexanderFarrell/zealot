import {
    BaseElementEmpty,
    CTRL_OR_META_KEY,
    Hotkey,
    ModalCommands,
    commands,
    registerNavigationCommands,
    registerToolCommands,
    setNavigator,
    setRightSidebarHost,
} from '@websoil/engine';
import { AddItemModal, ItemSearchModal, RightSidebar } from '@zealot/ui';
import '@zealot/ui/src/shell/mobile_title_bar';
import { MobileNavigator } from './mobile_navigator';

class MobileClient extends BaseElementEmpty {
    render() {
        const nav = new MobileNavigator();
        setNavigator(nav);
        commands.runner.clear();
        registerNavigationCommands();
        registerToolCommands();
        commands.runner.register(ModalCommands.newItem, [new Hotkey('n', [CTRL_OR_META_KEY])], () => {
            AddItemModal.show();
        });
        commands.runner.register(ModalCommands.openGlobalSearch, [new Hotkey('o', [CTRL_OR_META_KEY])], () => {
            ItemSearchModal.show();
        });

        this.innerHTML = `
        <mobile-title-bar></mobile-title-bar>
        <main id="main_mobile_ui">
            <center-content></center-content>
            <right-sidebar></right-sidebar>
        </main>
        `;

        setRightSidebarHost(document.querySelector('right-sidebar') as RightSidebar);
        nav.resolve();
    }
}

customElements.define('zealot-mobile-client', MobileClient);

export { MobileClient };
