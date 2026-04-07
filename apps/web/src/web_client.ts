import { BaseElementEmpty } from "@websoil/engine";
import { default_side_button_entries, SideButtons } from "@zealot/ui";

// Just for the UI itself. Much should come from shared components.
class ZealotWebClient extends BaseElementEmpty {
    render() {
        // A lot should come from shared components.
        this.innerHTML = `
        <header-bar></header-bar>
        <main>
            <side-buttons class="desktop_only"></side-buttons>
            <side-bar id="left_side_bar"></side-bar>
            <center-content></center-content>
            <side-bar id="right_side_bar"></side-bar>
        </main>
        <footer-bar></footer-bar>
        `

        let side_buttons = new SideButtons().init(default_side_button_entries())
    }
}

customElements.define('zealot-web-client', ZealotWebClient);

export default ZealotWebClient;