import { BaseElement, ModalCommands, NavigationCommands, Popups, ToolCommands, commands } from "@websoil/engine";
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
    const showNotImplemented = () => Popups.add_warning('This action is not implemented yet.');
    let buttons: SideButtonEntry[] = [
        {
            Title: "Home Page",
            IconURL: icons.home,
            Command: NavigationCommands.goHome
        },
        {
            Title: "New Item",
            IconURL: icons.add,
            Command: ModalCommands.newItem,
        },
        null,
        {
            Title: "Search",
            IconURL: icons.search,
            Command: ToolCommands.searchItems
        },
        {
            Title: "Navigation Side View",
            IconURL: icons.tree2,
            Command: ToolCommands.openNavTree
        },
        {
            Title: "Calendar",
            IconURL: icons.calendar,
            Command: ToolCommands.openCalendar
        },
        null,
        {
            Title: "Daily Planner",
            IconURL: icons.today,
            Command: NavigationCommands.openDailyPlanner
        },
        {
            Title: "Weekly Planner",
            IconURL: icons.week,
            Command: NavigationCommands.openWeeklyPlanner
        },
        {
            Title: "Monthly Planner",
            IconURL: icons.moon,
            Command: NavigationCommands.openMonthlyPlanner
        },
        {
            Title: "Annual Planner",
            IconURL: icons.sun,
            Command: NavigationCommands.openAnnualPlanner
        },
        null,
        {
            Title: "Types",
            IconURL: icons.table,
            Command: NavigationCommands.openTypes
        },
        {
            Title: "Analysis",
            IconURL: icons.science,
            Command: NavigationCommands.openAnalysis
        },
        {
            Title: "Rules",
            IconURL: icons.rules,
            Command: NavigationCommands.openRules
        },
        {
            Title: "Media",
            IconURL: icons.items,
            Command: NavigationCommands.openMedia
        },
        {
            Title: "Settings",
            IconURL: icons.settings,
            Command: NavigationCommands.openSettings
        },
        {
            Title: "Logout",
            IconURL: icons.logout,
            On: showNotImplemented,
        }
    ];
    return buttons
}

customElements.define('side-buttons', SideButtons);
customElements.define('side-button', SideButton);
