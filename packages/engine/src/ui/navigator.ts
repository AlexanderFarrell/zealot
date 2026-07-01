import { DateTime } from 'luxon';
import runner from './commands';
import { Hotkey, CTRL_OR_META_KEY, SHIFT_KEY, ALT_KEY } from './hotkeys';

export type SettingsSection = 'attributes' | 'types' | 'planner' | 'wiki' | 'data' | 'user';
export type PlannerView = 'daily' | 'weekly' | 'monthly' | 'annual';
export type TimeBlockView = 'day' | '3day' | 'week' | 'item';
export type AppLocation =
    | { kind: 'home' }
    | { kind: 'item'; itemId?: number; title?: string }
    | { kind: 'planner'; view: PlannerView; date: string }
    | { kind: 'media'; path: string }
    | { kind: 'types' }
    | { kind: 'type'; title: string }
    | { kind: 'analysis' }
    | { kind: 'analysis_specify' }
    | { kind: 'analysis_working' }
    | { kind: 'analysis_recent' }
    | { kind: 'analysis_backlog' }
    | { kind: 'analysis_overdue' }
    | { kind: 'analysis_most_viewed' }
    | { kind: 'rules' }
    | { kind: 'settings'; section: SettingsSection }
    | { kind: 'time_blocks'; view: TimeBlockView; date: string }
    | { kind: 'not_found'; path: string };

export type LocationListener = (location: AppLocation) => void;

export const NavigationCommands = {
    goHome: 'Go to Home',
    openDailyPlanner: 'Planner: Today',
    openWeeklyPlanner: 'Planner: This Week',
    openMonthlyPlanner: 'Planner: This Month',
    openAnnualPlanner: 'Planner: This Year',
    openTypes: 'Open Types',
    openAnalysis: 'Open Analysis',
    openRules: 'Open Rules',
    openSettings: 'Open Settings',
    openTodayNote: 'Open Today\'s Note',
    openMedia: 'Open Media',
    openRandomItem: 'Open Random Item',
} as const;

export interface Navigator {
    openHome(): void;
    openItem(title: string): void;
    openItemById(id: number): void;
    openMedia(path: string): void;
    openPlanner(view: PlannerView, date?: string): void;
    openTypes(): void;
    openType(title: string, mode?: 'push' | 'replace'): void;
    openAnalysis(): void;
    openAnalysisSpecify(): void;
    openAnalysisWorking(): void;
    openAnalysisRecent(): void;
    openAnalysisBacklog(): void;
    openAnalysisOverdue(): void;
    openAnalysisMostViewed(): void;
    openRules(): void;
    openSettings(section?: SettingsSection): void;
    openTimeBlocks(view?: TimeBlockView, date?: string): void;
    openTimeBlocksForItem(itemId: number): void;
    getLocation(): AppLocation;
    subscribe(listener: LocationListener): () => void;
    resolve(): void;
}

let _navigator: Navigator | null = null;

export function setNavigator(nav: Navigator): void {
    _navigator = nav;
}

export function getNavigator(): Navigator {
    if (!_navigator) {
        throw new Error('Navigator not registered');
    }
    return _navigator;
}

let _openInNewTabHandler: ((path: string) => void) | null = null;

export function setOpenInNewTabHandler(fn: (path: string) => void): void {
    _openInNewTabHandler = fn;
}

export function openInNewTab(path: string): void {
    if (_openInNewTabHandler) {
        _openInNewTabHandler(path);
    } else {
        window.open(path, '_blank');
    }
}

export function registerNavigationCommands(): void {
    runner.register(NavigationCommands.goHome, [new Hotkey('h', [CTRL_OR_META_KEY])], () => getNavigator().openHome());
    runner.register(NavigationCommands.openDailyPlanner, [new Hotkey('1', [CTRL_OR_META_KEY])], () => getNavigator().openPlanner('daily'));
    runner.register(NavigationCommands.openWeeklyPlanner, [new Hotkey('2', [CTRL_OR_META_KEY])], () => getNavigator().openPlanner('weekly'));
    runner.register(NavigationCommands.openMonthlyPlanner, [new Hotkey('3', [CTRL_OR_META_KEY])], () => getNavigator().openPlanner('monthly'));
    runner.register(NavigationCommands.openAnnualPlanner, [new Hotkey('4', [CTRL_OR_META_KEY])], () => getNavigator().openPlanner('annual'));
    runner.register(NavigationCommands.openTypes, [new Hotkey('t', [CTRL_OR_META_KEY, SHIFT_KEY, ALT_KEY])], () => getNavigator().openTypes());
    runner.register(NavigationCommands.openAnalysis, [new Hotkey('1', [CTRL_OR_META_KEY, SHIFT_KEY])], () => getNavigator().openAnalysis());
    runner.register('Analysis: Specify', [], () => getNavigator().openAnalysisSpecify());
    runner.register('Analysis: Working', [], () => getNavigator().openAnalysisWorking());
    runner.register('Analysis: Recent', [new Hotkey('r', [CTRL_OR_META_KEY, SHIFT_KEY])], () => getNavigator().openAnalysisRecent());
    runner.register('Analysis: Backlog', [], () => getNavigator().openAnalysisBacklog());
    runner.register('Analysis: Overdue', [], () => getNavigator().openAnalysisOverdue());
    runner.register('Analysis: Most Viewed', [], () => getNavigator().openAnalysisMostViewed());
    runner.register(NavigationCommands.openRules, [new Hotkey('2', [CTRL_OR_META_KEY, SHIFT_KEY])], () => getNavigator().openRules());
    runner.register(NavigationCommands.openSettings, [new Hotkey('3', [CTRL_OR_META_KEY, SHIFT_KEY])], () => getNavigator().openSettings());
    runner.register('Settings: Attributes', [], () => getNavigator().openSettings('attributes'));
    runner.register('Settings: Types', [], () => getNavigator().openSettings('types'));
    runner.register('Settings: Planner', [], () => getNavigator().openSettings('planner'));
    runner.register('Settings: Wiki', [], () => getNavigator().openSettings('wiki'));
    runner.register('Settings: Data', [], () => getNavigator().openSettings('data'));
    runner.register('Settings: User', [], () => getNavigator().openSettings('user'));
    runner.register(NavigationCommands.openTodayNote, [new Hotkey('d', [CTRL_OR_META_KEY, SHIFT_KEY])], () => getNavigator().openItem(DateTime.local().toISODate() ?? DateTime.local().toFormat('yyyy-MM-dd')));
    runner.register(NavigationCommands.openMedia, [new Hotkey('m', [CTRL_OR_META_KEY])], () => getNavigator().openMedia(''));
    runner.register('Time Blocks: Today', [], () => getNavigator().openTimeBlocks('day'));
    runner.register('Time Blocks: This Week', [], () => getNavigator().openTimeBlocks('week'));
}
