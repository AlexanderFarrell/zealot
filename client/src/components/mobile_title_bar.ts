import HomeIcon from '../assets/icon/home.svg';
import ZealotIcon from '../../public/zealot.webp';
import NavIcon from '../assets/icon/tree2.svg';
import SearchIcon from '../assets/icon/search.svg';
import CalendarIcon from "../assets/icon/month.svg";
import DailyIcon from "../assets/icon/today.svg";
import WeeklyIcon from "../assets/icon/week.svg";
import MonthlyIcon from "../assets/icon/moon.svg";
import AnnualIcon from "../assets/icon/sun.svg";
import AnalysisIcon from "../assets/icon/science.svg";
import RulesIcon from "../assets/icon/rules.svg";
import MediaIcon from "../assets/icon/items.svg";
import SettingsIcon from "../assets/icon/settings.svg";
import NewIcon from "../assets/icon/add.svg";
import LogoutIcon from "../assets/icon/logout.svg";
import HamburgerIcon from '../assets/icon/hamburger.svg';
import { router } from '../core/router';
import commands from "../core/command_runner.ts";
import BackButton from "../assets/icon/back.svg";
import { BaseElementEmpty } from './common/base_element.ts';
import { AddIcon, RunIcon, TableIcon } from '../assets/asset_map.ts';
import SearchView from './sidebars/search_view.ts';

// let mobile_menu_dropdown: MobileDropdown | null = null;
// let mobile_menu_toggled = false;

class MobileDropdown extends BaseElementEmpty {
    render() {
        let buttons = [
            {
                name: "Home",
                command: "Go to Home Page",
                icon: HomeIcon
            },
            {
                name: "Daily Planner",
                command: "Open Daily Planner",
                icon: DailyIcon
            },
            {
                name: "Weekly Planner",
                command: "Open Weekly Planner",
                icon: WeeklyIcon
            },
            {
                name: "Monthly Planner",
                command: "Open Monthly Planner",
                icon: MonthlyIcon
            },
            {
                name: "Annual Planner",
                command: "Open Annual Planner",
                icon: AnnualIcon
            },
            {
                name: "Analysis",
                command: "Open Analysis",
                icon: AnalysisIcon
            },
            {
                name: "Rules",
                command: "Open Rules Editor",
                icon: RulesIcon
            },
            {
                name: "Media",
                command: "Open Media",
                icon: MediaIcon
            },
            {
                name: "Settings",
                command: "Open Settings",
                icon: SettingsIcon
            },
            {
                name: "Logout",
                command: "Logout",
                icon: LogoutIcon
            },
            {
                name: "Types",
                command: "Open Item Types",
                icon: TableIcon
            },
            {
                name: "Back",
                command: null,
                icon: BackButton
            }
        ]

        buttons.forEach(data => {
            let button = document.createElement('button');
            button.innerHTML = `
                <img src="${data.icon}" style="width: 3.5em">
                <div>${data.name}</div>
            `
            button.addEventListener('click', () => {
                if (data.command != null) {
                    commands.run(data.command!);
                }
                this.remove();
                // mobile_menu_toggled = false;
            })
            this.appendChild(button);
        })
    }
}

class MobileTitleBar extends BaseElementEmpty {
    render() {
        this.classList.add('box')
        // this.classList.add('mobile_only')
        this.innerHTML = `
            <div style="display: grid; grid-template-columns: auto 1fr auto auto auto auto auto" id="mobile_title_bar_top">
                <button name='Home' style="background: none;"><img style="width: 2.5em" src="${ZealotIcon}"></button>
                <div>&nbsp;</div>
                <button name="Add" style="background: none;">
                    <img style="width: 2.5em" src="${AddIcon}">
                </button>
                <button name="Search" style="background: none;">
                    <img style="width: 2.5em" src="${SearchIcon}">
                </button>
                <button name="Command" style="background: none;">
                    <img style="width: 2.5em" src="${RunIcon}">
                </button>
                <button name="Daily" style="background: none;">
                    <img style="width: 2.5em" src="${DailyIcon}">
                </button>
                <button name='Hamburger' style="background: none;">
                    <img style="width: 2.5em" src="${HamburgerIcon}">
                </button>
            </div>
            <div name="open_screen">
            </div>

        `

        let screen_container = this.querySelector('[name="open_screen"]')! as HTMLDivElement;

        let home = (this.querySelector('[name="Home"]')! as HTMLButtonElement);
        home.addEventListener('click', () => {
            router.navigate('/');
        })
        let hamburger = (this.querySelector('[name="Hamburger"]')! as HTMLButtonElement);
        hamburger.addEventListener('click', () => {
            let mobile_dropdown = this.querySelector('mobile-dropdown')!;
            if (mobile_dropdown != null) {
                mobile_dropdown.remove();
            } else {
                screen_container.innerHTML = "";
                screen_container.appendChild(new MobileDropdown());
            }
        });

        let add_button = this.querySelector('[name="Add"]')! as HTMLButtonElement;
        add_button.addEventListener('click', () => {
            commands.run("New Item")
        })

        let daily_button = this.querySelector('[name="Daily"]')! as HTMLButtonElement
        daily_button.addEventListener('click', () => {
            commands.run("Open Daily Planner")
        })

        let run_button = this.querySelector('[name="Command"]')! as HTMLButtonElement;
        run_button.addEventListener('click', () => {
            commands.run("Open Command Runner")
        })

        let search_button = this.querySelector('[name="Search"]')! as HTMLButtonElement;
        search_button.addEventListener('click', () => {
            let search = this.querySelector('search-view')! as SearchView;
            if (search != null) {
                search.remove();
            } else {
                screen_container.innerHTML = "";
                search = new SearchView();
                search.dismiss_on_click = true;
                screen_container.appendChild(search)
            }
        })
    }
}

customElements.define('mobile-title-bar', MobileTitleBar);
customElements.define('mobile-dropdown', MobileDropdown);

export default MobileTitleBar;