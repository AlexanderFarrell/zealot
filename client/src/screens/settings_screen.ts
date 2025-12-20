import { router } from "../core/router";
import AttributeSettingsScreen from "./settings/attr_settings_screen";
import DataSettingsScreen from "./settings/data_settings_screen";
import PlannerSettingsScreen from "./settings/planner_settings_screen";
import TypeSettingsScreen from "./settings/type_settings_screen";
import UserSettingsScreen from "./settings/user_settings_screen";
import WikiSettingsScreen from "./settings/wiki_settings_screen";

class SettingsScreen extends HTMLElement {
    private current_screen: string = "attributes";
    private screen_container: HTMLElement | null = null;

    connectedCallback() {
        this.innerHTML = `
        <div id="row_buttons" class="row gap"></div>
        <div id="screen_container"></div>
        `
        let row_buttons = this.querySelector("#row_buttons");
        this.screen_container = this.querySelector('#screen_container');

        let button_names = ["attributes", "types", "planner", "wiki", "data", "user"];
        button_names.forEach(name => {
            let button = document.createElement('button');
            let capitalized_name = name.charAt(0).toUpperCase() + name.slice(1);
            button.innerHTML = capitalized_name;
            button.addEventListener('click', () => {
                router.navigate(`/settings/${name}`)
            })
            row_buttons?.appendChild(button);
        })

        this.switch_screen("attributes");
    }

    disconnectedCallback() {
        
    }

    switch_screen(next_screen: string) {
        let element = null;
        switch(next_screen) {
            case "attributes":
                element = new AttributeSettingsScreen();
                break;
            case "types":
                element = new TypeSettingsScreen();
                break;
            case "planner":
                element = new PlannerSettingsScreen();
                break;
            case "wiki":
                element = new WikiSettingsScreen();
                break;
            case "data":
                element = new DataSettingsScreen();
                break;
            case "user":
                element = new UserSettingsScreen();
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