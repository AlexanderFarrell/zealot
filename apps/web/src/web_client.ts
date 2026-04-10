import { BaseElementEmpty, setNavigator, registerNavigationCommands } from "@websoil/engine";
import { default_side_button_entries, SideButtons } from "@zealot/ui";
import { WebNavigator } from "./web_navigator";

// Just for the UI itself. Much should come from shared components.
class ZealotWebClient extends BaseElementEmpty {
    render() {
        // A lot should come from shared components.
        this.innerHTML = `
        <header-bar></header-bar>
        <main id="main_web_ui">
            <side-buttons class="desktop_only"></side-buttons>
            <side-bar id="left_side_bar">a</side-bar>
            <center-content>a</center-content>
            <side-bar id="right_side_bar">a</side-bar>
        </main>
        <footer-bar></footer-bar>
        `;

        (document.querySelector("side-buttons")! as SideButtons).init(default_side_button_entries());

        const nav = new WebNavigator();
        setNavigator(nav);
        registerNavigationCommands();
        nav.resolve();
    }
}

customElements.define('zealot-web-client', ZealotWebClient);

export default ZealotWebClient;