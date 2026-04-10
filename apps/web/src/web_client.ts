import {
    BaseElementEmpty,
    CTRL_OR_META_KEY,
    Hotkey,
    ModalCommands,
    commands,
    registerNavigationCommands,
    registerToolCommands,
    setNavigator,
    setToolHost,
} from "@websoil/engine";
import { AddItemModal, ItemSearchModal, default_side_button_entries, SideButtons } from "@zealot/ui";
import "@zealot/ui/src/shell/mobile_title_bar";
import { WebNavigator } from "./web_navigator";
import "./web_tool_host";
import { WebToolHost } from "./web_tool_host";

class ZealotWebClient extends BaseElementEmpty {
    render() {
        const nav = new WebNavigator();
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
        <header-bar class="desktop_only"></header-bar>
        <mobile-title-bar class="mobile_only"></mobile-title-bar>
        <main id="main_web_ui">
            <side-buttons class="desktop_only"></side-buttons>
            <web-tool-host id="left_tool_host" class="desktop_only"></web-tool-host>
            <center-content></center-content>
        </main>
        <footer-bar></footer-bar>
        `;

        (document.querySelector("side-buttons")! as SideButtons).init(default_side_button_entries());
        setToolHost(document.querySelector('web-tool-host')! as WebToolHost);

        nav.resolve();
    }
}

customElements.define('zealot-web-client', ZealotWebClient);

export default ZealotWebClient;
