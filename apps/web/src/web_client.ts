import { BaseElementEmpty, commands, registerNavigationCommands, registerToolCommands, setNavigator, setToolHost } from "@websoil/engine";
import { default_side_button_entries, SideButtons } from "@zealot/ui";
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

        this.innerHTML = `
        <header-bar></header-bar>
        <main id="main_web_ui">
            <side-buttons class="desktop_only"></side-buttons>
            <web-tool-host id="left_tool_host"></web-tool-host>
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
