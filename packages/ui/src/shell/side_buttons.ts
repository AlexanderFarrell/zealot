import { BaseElement, BaseElementEmpty, commands } from "@websoil/engine";
import { icons } from "@zealot/content";

interface SideButtonInfo {
    Title: string,
    IconURL: string,
    Command?: string,
    On?: () => void,
}

type SideButtonEntry = SideButtonInfo | null

class SideButton extends BaseElement<SideButtonInfo> {
    render() {
        this.innerHTML = `
        <img src="${this.data!.IconURL}">
        `
        this.title = this.data!.Title
        this.addEventListener('click', () => {
            if (this.data!.Command) {
                commands.runner.run(this.data!.Command!)
            } else if (this.data!.On) {
                this.data!.On()
            }
        })
    }
}

export class SideButtons extends BaseElement<SideButtonEntry[]> {
    async render() {
        this.data!.forEach(entry => {
            if (entry != null) {
                this.appendChild(new SideButton().init(entry!))
            } else {
                this.appendChild(document.createElement('hr'));
            }
        })
    }
}

export function default_side_button_entries(): SideButtonEntry[] {
    let buttons: SideButtonEntry[] = [
        {
            Title: "Home Page",
            IconURL: icons.home,
            Command: "Go to Home Page"
        },
        {
            Title: "New Item",
            IconURL: icons.add,
            Command: "New Item"
        },
        null,
        {
            Title: "Search",
            IconURL: icons.search,
            Command: "Search Items"
        },
        {
            Title: "Navigation Side View",
            IconURL: icons.tree2,
            Command: "Open Nav Sidebar"
        },
        {
            Title: "Calendar",
            IconURL: icons.calendar,
            Command: "Open Calendar"
        },
        null,
        {
            Title: "Daily Planner",
            IconURL: icons.today,
            Command: "Open Daily Planner"
        },
        {
            Title: "Weekly Planner",
            IconURL: icons.week,
            Command: "Open Weekly Planner"
        },
        {
            Title: "Monthly Planner",
            IconURL: icons.moon,
            Command: "Open Monthly Planner"
        },
        {
            Title: "Annual Planner",
            IconURL: icons.sun,
            Command: "Open Annual Planner"
        },
        null,
        {
            Title: "Types",
            IconURL: icons.table,
            Command: "Open Item Types"
        },
        {
            Title: "Analysis",
            IconURL: icons.science,
            Command: "Open Analysis"
        },
        {
            Title: "Rules",
            IconURL: icons.rules,
            Command: "Open Rules Editor"
        },
        {
            Title: "Media",
            IconURL: icons.items,
            Command: "Open Media"
        },
        {
            Title: "Settings",
            IconURL: icons.settings,
            Command: "Open Settings"
        },
        {
            Title: "Logout",
            IconURL: icons.logout,
            Command: "Logout"
        }
    ];
    return buttons
}

customElements.define('side-buttons', SideButtons);
customElements.define('side-button', SideButton);