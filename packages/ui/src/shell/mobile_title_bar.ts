import { BaseElementEmpty, ModalCommands, NavigationCommands, commands, getNavigator } from "@websoil/engine";
import { ZealotIcon, icons } from "@zealot/content";
import { CommandPaletteModal } from '../common/command_palette_modal';

interface DropdownEntry {
    name: string;
    command: string;
    iconURL: string;
}

const DROPDOWN_ENTRIES: DropdownEntry[] = [
    { name: 'Home',             command: NavigationCommands.goHome,           iconURL: icons.home },
    { name: 'Daily Planner',    command: NavigationCommands.openDailyPlanner, iconURL: icons.today },
    { name: 'Weekly Planner',   command: NavigationCommands.openWeeklyPlanner,iconURL: icons.week },
    { name: 'Monthly Planner',  command: NavigationCommands.openMonthlyPlanner,iconURL: icons.moon },
    { name: 'Annual Planner',   command: NavigationCommands.openAnnualPlanner, iconURL: icons.sun },
    { name: 'Analysis',         command: NavigationCommands.openAnalysis,      iconURL: icons.science },
    { name: 'Rules',            command: NavigationCommands.openRules,         iconURL: icons.rules },
    { name: 'Media',            command: NavigationCommands.openMedia,         iconURL: icons.items },
    { name: 'Types',            command: NavigationCommands.openTypes,         iconURL: icons.table },
    { name: 'Settings',         command: NavigationCommands.openSettings,      iconURL: icons.settings },
];

export class MobileTitleBar extends BaseElementEmpty {
    private dropdownOpen = false;
    private dropdownEl: HTMLElement | null = null;
    private navDepth = 0;
    private unsubscribe: (() => void) | null = null;

    render() {
        this.innerHTML = `
        <div id="mobile_title_bar_top">
            <button name="back" style="display:none"><img src="${icons.back}" alt="Back"></button>
            <button name="home"><img src="${ZealotIcon}" alt="Zealot"></button>
            <div class="header-bar-spacer"></div>
            <button name="commands"><img src="${icons.bolt}" alt="Commands"></button>
            <button name="add"><img src="${icons.add}" alt="New Item"></button>
            <button name="search"><img src="${icons.search}" alt="Search"></button>
            <button name="daily"><img src="${icons.today}" alt="Daily"></button>
            <button name="hamburger"><img src="${icons.hamburger}" alt="Menu"></button>
        </div>
        <div name="dropdown_container"></div>
        `;

        this.querySelector<HTMLButtonElement>('[name="back"]')!
            .addEventListener('click', () => {
                if (this.navDepth > 0) history.back();
            });

        this.querySelector<HTMLButtonElement>('[name="home"]')!
            .addEventListener('click', () => commands.runner.run(NavigationCommands.goHome));

        this.querySelector<HTMLButtonElement>('[name="commands"]')!
            .addEventListener('click', () => CommandPaletteModal.show());

        this.querySelector<HTMLButtonElement>('[name="add"]')!
            .addEventListener('click', () => commands.runner.run(ModalCommands.newItem));

        this.querySelector<HTMLButtonElement>('[name="search"]')!
            .addEventListener('click', () => commands.runner.run(ModalCommands.openGlobalSearch));

        this.querySelector<HTMLButtonElement>('[name="daily"]')!
            .addEventListener('click', () => commands.runner.run(NavigationCommands.openDailyPlanner));

        this.querySelector<HTMLButtonElement>('[name="hamburger"]')!
            .addEventListener('click', () => this.toggleDropdown());

        // Subscribe to location changes so we can show/hide the back button.
        try {
            this.unsubscribe = getNavigator().subscribe(() => {
                this.navDepth++;
                this.updateBackButton();
            });
        } catch {
            // Navigator may not be set yet during early render; that's fine.
        }
    }

    disconnectedCallback(): void {
        this.unsubscribe?.();
        this.unsubscribe = null;
    }

    private updateBackButton(): void {
        const btn = this.querySelector<HTMLButtonElement>('[name="back"]');
        if (btn) btn.style.display = this.navDepth > 0 ? '' : 'none';
    }

    toggleDropdown() {
        if (this.dropdownOpen) {
            this.closeDropdown();
        } else {
            this.openDropdown();
        }
    }

    private openDropdown() {
        const container = this.querySelector<HTMLElement>('[name="dropdown_container"]')!;
        container.innerHTML = '';

        const dropdown = document.createElement('div');
        dropdown.className = 'mobile-dropdown';

        // In tauri-app mode the bar is at the bottom, so the dropdown should open upward.
        if (document.body.classList.contains('tauri-app')) {
            dropdown.style.position = 'absolute';
            dropdown.style.bottom = '100%';
            dropdown.style.left = '0';
            dropdown.style.right = '0';
            dropdown.style.background = 'var(--bg-2)';
            dropdown.style.boxShadow = '0 -2px 8px var(--shadow-color-2)';
        }

        DROPDOWN_ENTRIES.forEach(entry => {
            const btn = document.createElement('button');
            btn.innerHTML = `<img src="${entry.iconURL}" alt="${entry.name}"><div>${entry.name}</div>`;
            btn.addEventListener('click', () => {
                commands.runner.run(entry.command);
                this.closeDropdown();
            });
            dropdown.appendChild(btn);
        });

        container.appendChild(dropdown);
        this.dropdownEl = dropdown;
        this.dropdownOpen = true;
    }

    private closeDropdown() {
        const container = this.querySelector<HTMLElement>('[name="dropdown_container"]')!;
        container.innerHTML = '';
        this.dropdownEl = null;
        this.dropdownOpen = false;
    }
}

customElements.define('mobile-title-bar', MobileTitleBar);
