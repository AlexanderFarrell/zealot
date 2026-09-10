import { DateTime } from 'luxon';
import {
    BaseElement, Events, ModalCommands, NavigationCommands, Popups, ToolCommands, commands,
    registerContextMenu, unregisterContextMenu,
    registerDropZone, unregisterDropZone,
    getNavigator, openInNewTab,
} from '@websoil/engine';
import type { ContextAction } from '@websoil/engine';
import { AttributeAPI } from '@zealot/api/src/attribute';
import { AuthAPI } from '@zealot/api/src/auth';
import { icons } from '@zealot/content';

const attrApi = new AttributeAPI('/api');
const authApi = new AuthAPI('/api');
let loggingOut = false;

const todayIso    = () => DateTime.local().toISODate()!;
const thisWeekIso = () => DateTime.local().toFormat("kkkk-'W'WW");
const thisMonthIso = () => DateTime.local().toFormat('yyyy-MM');
const thisYearStr  = () => String(DateTime.local().year);

interface SideButtonInfo {
    Title: string;
    IconURL: string;
    Command?: string;
    On?: () => void;
    Url?: () => string;
    ContextActions?: () => ContextAction[];
    DropAction?: (itemId: number) => void;
}

type SideButtonEntry = SideButtonInfo | null;

class SideButton extends BaseElement<SideButtonInfo> {
    render() {
        this.innerHTML = `<img src="${this.data!.IconURL}">`;
        this.title = this.data!.Title;

        this.addEventListener('click', () => {
            if (this.data!.Command) {
                commands.runner.run(this.data!.Command!);
            } else if (this.data!.On) {
                this.data!.On();
            }
        });

        const info = this.data!;

        // Context menu
        unregisterContextMenu(this);
        const baseActions: ContextAction[] = [];
        if (info.Url) {
            const url = info.Url();
            baseActions.push(
                { label: 'Open in New Tab',    onClick: () => openInNewTab(url) },
                { label: 'Open in New Window', onClick: () => window.open(url, '_blank', 'noopener,noreferrer') },
            );
        }
        const extras = info.ContextActions?.() ?? [];
        const allActions: ContextAction[] = extras.length > 0
            ? [...baseActions, { separator: true }, ...extras]
            : baseActions;
        if (allActions.length > 0) {
            registerContextMenu(this, () => allActions);
        }

        // Drop zone
        unregisterDropZone(this);
        if (info.DropAction) {
            registerDropZone({
                element: this,
                onDragOver: () => this.classList.add('side-button--drop-target'),
                onDrop: (id) => {
                    this.classList.remove('side-button--drop-target');
                    info.DropAction!(id);
                },
                onDragLeave: () => this.classList.remove('side-button--drop-target'),
            });
        }
    }
}

export class SideButtons extends BaseElement<SideButtonEntry[]> {
    async render() {
        this.data!.forEach(entry => {
            if (entry != null) {
                this.appendChild(new SideButton().init(entry!));
            } else {
                this.appendChild(document.createElement('hr'));
            }
        });
    }
}

export function default_side_button_entries(): SideButtonEntry[] {
    const logout = async () => {
        if (loggingOut) return;
        loggingOut = true;
        try {
            await authApi.Basic.logout();
            Events.emit('to_auth');
        } catch (error) {
            Popups.add_error(error instanceof Error && error.message ? error.message : 'Logout failed.');
        } finally {
            loggingOut = false;
        }
    };

    const buttons: SideButtonEntry[] = [
        {
            Title: "Home Page",
            IconURL: icons.home,
            Command: NavigationCommands.goHome,
            Url: () => '/',
        },
        {
            Title: "New Item",
            IconURL: icons.add,
            Command: ModalCommands.newItem,
        },
        {
            Title: "Random Items",
            IconURL: icons.chess,
            Command: ToolCommands.openRandom,
        },
        null,
        {
            Title: "Search",
            IconURL: icons.search,
            Command: ToolCommands.searchItems,
        },
        {
            Title: "Navigation Side View",
            IconURL: icons.tree2,
            Command: ToolCommands.openNavTree,
        },
        {
            Title: "Calendar",
            IconURL: icons.calendar,
            Command: ToolCommands.openCalendar,
        },
        null,
        {
            Title: "Daily Planner",
            IconURL: icons.today,
            Command: NavigationCommands.openDailyPlanner,
            Url: () => `/planner/daily/${todayIso()}`,
            ContextActions: () => [
                { label: "Yesterday", onClick: () => getNavigator().openPlanner('daily', DateTime.local().minus({ days: 1 }).toISODate()!) },
                { label: "Tomorrow",  onClick: () => getNavigator().openPlanner('daily', DateTime.local().plus({ days: 1 }).toISODate()!) },
            ],
            DropAction: (id) => { void attrApi.set_value(id, 'Date', todayIso()); },
        },
        {
            Title: "Weekly Planner",
            IconURL: icons.week,
            Command: NavigationCommands.openWeeklyPlanner,
            Url: () => `/planner/weekly/${thisWeekIso()}`,
            ContextActions: () => [
                { label: "Last Week", onClick: () => getNavigator().openPlanner('weekly', DateTime.local().minus({ weeks: 1 }).toFormat("kkkk-'W'WW")) },
                { label: "Next Week", onClick: () => getNavigator().openPlanner('weekly', DateTime.local().plus({ weeks: 1 }).toFormat("kkkk-'W'WW")) },
            ],
            DropAction: (id) => { void attrApi.set_value(id, 'Week', thisWeekIso()); },
        },
        {
            Title: "Monthly Planner",
            IconURL: icons.moon,
            Command: NavigationCommands.openMonthlyPlanner,
            Url: () => `/planner/monthly/${thisMonthIso()}`,
            ContextActions: () => [
                { label: "Last Month", onClick: () => getNavigator().openPlanner('monthly', DateTime.local().minus({ months: 1 }).toFormat('yyyy-MM')) },
                { label: "Next Month", onClick: () => getNavigator().openPlanner('monthly', DateTime.local().plus({ months: 1 }).toFormat('yyyy-MM')) },
            ],
            DropAction: (id) => {
                const now = DateTime.local();
                void attrApi.set_value(id, 'Month', now.month);
                void attrApi.set_value(id, 'Year', now.year);
            },
        },
        {
            Title: "Annual Planner",
            IconURL: icons.sun,
            Command: NavigationCommands.openAnnualPlanner,
            Url: () => `/planner/annual/${thisYearStr()}`,
            ContextActions: () => [
                { label: "Last Year", onClick: () => getNavigator().openPlanner('annual', String(DateTime.local().year - 1)) },
                { label: "Next Year", onClick: () => getNavigator().openPlanner('annual', String(DateTime.local().year + 1)) },
            ],
            DropAction: (id) => { void attrApi.set_value(id, 'Year', DateTime.local().year); },
        },
        null,
        {
            Title: "Types",
            IconURL: icons.table,
            Command: NavigationCommands.openTypes,
            Url: () => '/types',
        },
        {
            Title: "Analysis",
            IconURL: icons.science,
            Command: NavigationCommands.openAnalysis,
            Url: () => '/analysis',
            ContextActions: () => [
                { label: "Specify",     onClick: () => getNavigator().openAnalysisSpecify() },
                { label: "Working",     onClick: () => getNavigator().openAnalysisWorking() },
                { label: "Recent",      onClick: () => getNavigator().openAnalysisRecent() },
                { label: "Backlog",     onClick: () => getNavigator().openAnalysisBacklog() },
                { label: "Overdue",     onClick: () => getNavigator().openAnalysisOverdue() },
                { label: "Most Viewed", onClick: () => getNavigator().openAnalysisMostViewed() },
            ],
        },
        {
            Title: "Rules",
            IconURL: icons.rules,
            Command: NavigationCommands.openRules,
            Url: () => '/rules',
        },
        {
            Title: "Media",
            IconURL: icons.items,
            Command: NavigationCommands.openMedia,
            Url: () => '/media',
        },
        {
            Title: "Settings",
            IconURL: icons.settings,
            Command: NavigationCommands.openSettings,
            Url: () => '/settings/attributes',
            ContextActions: () => [
                { label: "Attributes", onClick: () => getNavigator().openSettings('attributes') },
                { label: "Types",      onClick: () => getNavigator().openSettings('types') },
                { label: "Planner",    onClick: () => getNavigator().openSettings('planner') },
                { label: "Wiki",       onClick: () => getNavigator().openSettings('wiki') },
                { label: "Data",       onClick: () => getNavigator().openSettings('data') },
                { label: "User",       onClick: () => getNavigator().openSettings('user') },
            ],
        },
        {
            Title: "Logout",
            IconURL: icons.logout,
            On: () => { void logout(); },
        },
    ];

    return buttons;
}

customElements.define('side-buttons', SideButtons);
customElements.define('side-button', SideButton);
