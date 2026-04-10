import runner from './commands';
import { Hotkey, CTRL_OR_META_KEY, SHIFT_KEY, ALT_KEY } from './hotkeys';

export type SettingsSection = 'attributes' | 'types' | 'planner' | 'wiki' | 'data' | 'user';
export type PlannerView = 'daily' | 'weekly' | 'monthly' | 'annual';

export interface Navigator {
    openHome(): void;
    openItem(title: string): void;
    openItemById(id: number): void;
    openMedia(path: string): void;
    openPlanner(view: PlannerView, date?: string): void;
    openTypes(): void;
    openType(title: string): void;
    openAnalysis(): void;
    openRules(): void;
    openSettings(section?: SettingsSection): void;
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

export function registerNavigationCommands(): void {
    runner.register('Go to Home', [new Hotkey('h', [CTRL_OR_META_KEY])], () => getNavigator().openHome());
    runner.register('Open Daily Planner', [new Hotkey('1', [CTRL_OR_META_KEY])], () => getNavigator().openPlanner('daily'));
    runner.register('Open Weekly Planner', [new Hotkey('2', [CTRL_OR_META_KEY])], () => getNavigator().openPlanner('weekly'));
    runner.register('Open Monthly Planner', [new Hotkey('3', [CTRL_OR_META_KEY])], () => getNavigator().openPlanner('monthly'));
    runner.register('Open Annual Planner', [new Hotkey('4', [CTRL_OR_META_KEY])], () => getNavigator().openPlanner('annual'));
    runner.register('Open Types', [new Hotkey('t', [CTRL_OR_META_KEY, SHIFT_KEY, ALT_KEY])], () => getNavigator().openTypes());
    runner.register('Open Analysis', [new Hotkey('1', [CTRL_OR_META_KEY, SHIFT_KEY])], () => getNavigator().openAnalysis());
    runner.register('Open Rules', [new Hotkey('2', [CTRL_OR_META_KEY, SHIFT_KEY])], () => getNavigator().openRules());
    runner.register('Open Settings', [new Hotkey('3', [CTRL_OR_META_KEY, SHIFT_KEY])], () => getNavigator().openSettings());
    runner.register('Open Today\'s Note', [new Hotkey('d', [CTRL_OR_META_KEY, SHIFT_KEY])], () => getNavigator().openItem(new Date().toISOString().slice(0, 10)));
    runner.register('Open Media', [new Hotkey('m', [CTRL_OR_META_KEY])], () => getNavigator().openMedia(''));
    // Deferred: 'Search Items' (Ctrl+O), 'Open Command Runner' (Ctrl+P), 'New Item' (Ctrl+N) — modals not yet built
}
