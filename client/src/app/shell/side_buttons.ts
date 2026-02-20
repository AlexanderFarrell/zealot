import IconButton from '../../shared/icon_button.ts';
import commands from "../../shared/command_runner.ts";
import DragUtil from '../../features/item/drag_helper.ts';
import { DateTime } from 'luxon';
import ZealotIcon from '../../../public/zealot.webp';
import { HomeIcon, Tree2Icon, SearchIcon, MoonIcon, TodayIcon, WeekIcon, 
    SunIcon, ScienceIcon, MonthIcon, RulesIcon, ItemsIcon, SettingsIcon, AddIcon,
    HamburgerIcon, LogoutIcon, RunIcon, TableIcon, BackIcon, CalendarIcon } 
    from '../../assets/asset_map.ts';



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
                AddIcon,
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
                Tree2Icon,
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
                TodayIcon,
                "Daily Planner",
                () => {commands.run("Open Daily Planner")}
            ),
            new IconButton(
                WeekIcon,
                "Weekly Planner",
                () => {commands.run("Open Weekly Planner")}
            ),
            new IconButton(
                MoonIcon,
                "Monthly Planner",
                () => {commands.run("Open Monthly Planner")}
            ),

            new IconButton(
                SunIcon,
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
                ScienceIcon,
                "Analysis",
                () => {commands.run("Open Analysis")}
            ),
            new IconButton(
                RulesIcon,
                "Rules",
                () => {commands.run("Open Rules Editor")}
            ),
            new IconButton(
                ItemsIcon,
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


        // Set up drag
        DragUtil.setup_drop(this.querySelector('[title="Daily Planner"]')!, 
            {"Date": DateTime.now().toISODate()});
        DragUtil.setup_drop(this.querySelector('[title="Weekly Planner"]')!, 
            {"Week": DateTime.now().toISOWeekDate().substring(0, 8)});
        DragUtil.setup_drop(this.querySelector('[title="Monthly Planner"]')!, 
            {"Month": DateTime.now().month, "Year": DateTime.now().year});
        DragUtil.setup_drop(this.querySelector('[title="Annual Planner"]')!, 
            {"Year": DateTime.now().year});
    }
}

customElements.define('side-buttons', SideButtons)

export default SideButtons;