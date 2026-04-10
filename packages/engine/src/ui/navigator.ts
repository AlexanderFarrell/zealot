import runner from './commands';

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
    runner.register('Go to Home', [], () => getNavigator().openHome());
    runner.register('Open Daily Planner', [], () => getNavigator().openPlanner('daily'));
    runner.register('Open Weekly Planner', [], () => getNavigator().openPlanner('weekly'));
    runner.register('Open Monthly Planner', [], () => getNavigator().openPlanner('monthly'));
    runner.register('Open Annual Planner', [], () => getNavigator().openPlanner('annual'));
    runner.register('Open Types', [], () => getNavigator().openTypes());
    runner.register('Open Analysis', [], () => getNavigator().openAnalysis());
    runner.register('Open Rules', [], () => getNavigator().openRules());
    runner.register('Open Settings', [], () => getNavigator().openSettings());
}
