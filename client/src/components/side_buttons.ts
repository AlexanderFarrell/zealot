import IconButton from './common/icon_button.ts';
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

import commands from "../core/command_runner.ts";
import { TableIcon } from '../assets/asset_map.ts';

class SideButtons extends HTMLElement {

    connectedCallback() {
        let buttons: Array<IconButton | null> = [
             new IconButton(
                // ZealotIcon,
                HomeIcon,
                "Home Page",
                () => {commands.run("Go to Home Page")}
            ),
            new IconButton(
                NewIcon,
                "New Item",
                () => {commands.run("New Item")}
            ),
            null,
            new IconButton(
                SearchIcon,
                "Search",
                () => {commands.run("Search Items")}
            ),
            new IconButton(
                NavIcon,
                "Navigation Side View",
                () => {commands.run("Open Nav Sidebar")}
            ),

            new IconButton(
                CalendarIcon,
                "Calendar",
                () => {commands.run("Open Calendar")}
            ), 
            null,
            new IconButton(
                DailyIcon,
                "Daily Planner",
                () => {commands.run("Open Daily Planner")}
            ),
            new IconButton(
                WeeklyIcon,
                "Weekly Planner",
                () => {commands.run("Open Weekly Planner")}
            ),
            new IconButton(
                MonthlyIcon,
                "Monthly Planner",
                () => {commands.run("Open Monthly Planner")}
            ),

            new IconButton(
                AnnualIcon,
                "Annual Planner",
                () => {commands.run("Open Annual Planner")}
            ),
            null,

            new IconButton(
                TableIcon,
                "Types",
                () => {commands.run("Open Item Types")}
            ),

            new IconButton(
                AnalysisIcon,
                "Analysis",
                () => {commands.run("Open Analysis")}
            ),
            new IconButton(
                RulesIcon,
                "Rules",
                () => {commands.run("Open Rules Editor")}
            ),
            new IconButton(
                MediaIcon,
                "Media",
                () => {commands.run("Open Media")}
            ),
            new IconButton(
                SettingsIcon,
                "Settings",
                () => {commands.run("Open Settings")}
            ),

            new IconButton(
                LogoutIcon,
                "Logout",
                () => {commands.run("Logout")}
            ),
        ]

        buttons.forEach(button => {
            if (button != null) {
                this.appendChild(button!);
            } else {
                this.appendChild(document.createElement('hr'))
            }
        })
    }
}

customElements.define('side-buttons', SideButtons)

export default SideButtons;