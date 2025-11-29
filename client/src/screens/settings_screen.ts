import DataSettingsScreen from "./settings/data_settings_screen";
import PlannerSettingsScreen from "./settings/planner_settings_screen";
import TypeSettingsScreen from "./settings/type_settings_screen";
import WikiSettingsScreen from "./settings/wiki_settings_screen";

class SettingsScreen extends HTMLElement {
    private current_screen: string = "Types";
    private screen_container: HTMLElement | null = null;

    connectedCallback() {
        this.innerHTML = `
        <div id="row_buttons" class="row gap"></div>
        <div id="screen_container"></div>
        `
        let row_buttons = this.querySelector("#row_buttons");
        this.screen_container = this.querySelector('#screen_container');

        let button_names = ["Types", "Planner", "Wiki", "Data"];
        button_names.forEach(name => {
            let button = document.createElement('button');
            button.innerHTML = name;
            button.addEventListener('click', () => {
                this.switch_screen(name);
            })
            row_buttons?.appendChild(button);
        })

        this.switch_screen("Types");
    }

    disconnectedCallback() {

    }

    switch_screen(next_screen: string) {
        let element = null;
        switch(next_screen) {
            case "Types":
                element = new TypeSettingsScreen();
                break;
            case "Planner":
                element = new PlannerSettingsScreen();
                break;
            case "Wiki":
                element = new WikiSettingsScreen();
                break;
            case "Data":
                element = new DataSettingsScreen();
                break;
        }

        if (element != null) {
            this.screen_container!.innerHTML = "";
            this.screen_container!.appendChild(element);
        }

        this.current_screen = next_screen;
    }
}

customElements.define('settings-screen', SettingsScreen)

export default SettingsScreen;