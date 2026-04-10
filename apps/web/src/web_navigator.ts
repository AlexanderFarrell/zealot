import { DateTime } from 'luxon';
import type { AppLocation, LocationListener, Navigator, PlannerView, SettingsSection } from '@websoil/engine';
import { ItemScreen } from '@zealot/ui/src/screens/item_screen';
import { MediaScreen } from '@zealot/ui/src/screens/media_screen';
import { DailyPlannerScreen } from '@zealot/ui/src/screens/daily_planner_screen';
import { WeeklyPlannerScreen } from '@zealot/ui/src/screens/weekly_planner_screen';
import { MonthlyPlannerScreen } from '@zealot/ui/src/screens/monthly_planner_screen';
import { AnnualPlannerScreen } from '@zealot/ui/src/screens/annual_planner_screen';
import { TypesScreen } from '@zealot/ui/src/screens/types_screen';
import { TypeScreen } from '@zealot/ui/src/screens/type_screen';
import { AnalysisScreen } from '@zealot/ui/src/screens/analysis_screen';
import { RulesScreen } from '@zealot/ui/src/screens/rules_screen';
import { SettingsScreen } from '@zealot/ui/src/screens/settings_screen';

function todayIso(): string {
    return DateTime.local().toISODate() ?? DateTime.local().toFormat('yyyy-MM-dd');
}

function thisWeekIso(): string {
    return DateTime.local().toFormat("kkkk-'W'WW");
}

function thisMonthIso(): string {
    return DateTime.local().toFormat('yyyy-MM');
}

function thisYear(): string {
    return DateTime.local().toFormat('yyyy');
}

interface RouteMatch {
    screen: HTMLElement;
    location: AppLocation;
    redirectTo?: string;
}

export class WebNavigator implements Navigator {
    private location: AppLocation = { kind: 'not_found', path: window.location.pathname };
    private readonly listeners = new Set<LocationListener>();

    constructor() {
        window.addEventListener('popstate', () => this.renderCurrent());
    }

    private getContent(): HTMLElement {
        const el = document.querySelector('center-content');
        if (!el) throw new Error('No <center-content> element found');
        return el as HTMLElement;
    }

    private navigate(path: string, mode: 'push' | 'replace' = 'push'): void {
        if (mode === 'replace') {
            history.replaceState(null, '', path);
        } else {
            history.pushState(null, '', path);
        }
        this.renderCurrent();
    }

    private setLocation(location: AppLocation): void {
        this.location = location;
        this.listeners.forEach((listener) => listener(location));
    }

    openHome(): void {
        this.navigate('/');
    }

    openItem(title: string): void {
        this.navigate(`/item/${encodeURIComponent(title)}`);
    }

    openItemById(id: number): void {
        this.navigate(`/item_id/${id}`);
    }

    openMedia(path: string): void {
        this.navigate(path ? `/media/${path}` : '/media');
    }

    openPlanner(view: PlannerView, date?: string): void {
        switch (view) {
            case 'daily':
                this.navigate(`/planner/daily/${date ?? todayIso()}`);
                break;
            case 'weekly':
                this.navigate(`/planner/weekly/${date ?? thisWeekIso()}`);
                break;
            case 'monthly':
                this.navigate(`/planner/monthly/${date ?? thisMonthIso()}`);
                break;
            case 'annual':
                this.navigate(`/planner/annual/${date ?? thisYear()}`);
                break;
        }
    }

    openTypes(): void {
        this.navigate('/types');
    }

    openType(title: string, mode: 'push' | 'replace' = 'push'): void {
        this.navigate(`/types/${encodeURIComponent(title)}`, mode);
    }

    openAnalysis(): void {
        this.navigate('/analysis');
    }

    openRules(): void {
        this.navigate('/rules');
    }

    openSettings(section?: SettingsSection): void {
        this.navigate(`/settings/${section ?? 'attributes'}`);
    }

    getLocation(): AppLocation {
        return this.location;
    }

    subscribe(listener: LocationListener): () => void {
        this.listeners.add(listener);
        return () => this.listeners.delete(listener);
    }

    resolve(): void {
        this.renderCurrent();
    }

    private renderCurrent(): void {
        const path = window.location.pathname;
        const content = this.getContent();
        const route = this.matchRoute(path);
        if (route.redirectTo) {
            this.navigate(route.redirectTo, 'replace');
            return;
        }
        content.innerHTML = '';
        content.appendChild(route.screen);
        this.setLocation(route.location);
    }

    private matchRoute(path: string): RouteMatch {
        if (path === '/') {
            const s = new ItemScreen();
            s.loadItem('Home');
            return {
                screen: s,
                location: { kind: 'home' },
            };
        }
        if (path === '/analysis') {
            return {
                screen: new AnalysisScreen(),
                location: { kind: 'analysis' },
            };
        }
        if (path === '/rules') {
            return {
                screen: new RulesScreen(),
                location: { kind: 'rules' },
            };
        }
        if (path === '/types') {
            return {
                screen: new TypesScreen(),
                location: { kind: 'types' },
            };
        }

        const segments = path.split('/').filter(Boolean);

        if (segments[0] === 'item' && segments[1]) {
            const s = new ItemScreen();
            const title = decodeURIComponent(segments[1]);
            s.loadItem(title);
            return {
                screen: s,
                location: { kind: 'item', title },
            };
        }

        if (segments[0] === 'item_id' && segments[1]) {
            const id = parseInt(segments[1], 10);
            if (Number.isNaN(id)) {
                return this.notFound(path);
            }
            const s = new ItemScreen();
            s.loadItemById(id);
            return {
                screen: s,
                location: { kind: 'item', itemId: id },
            };
        }

        if (segments[0] === 'media') {
            const mediaPath = path === '/media' ? '' : path.replace(/^\/media\//, '');
            return {
                screen: new MediaScreen().init(mediaPath),
                location: { kind: 'media', path: mediaPath },
            };
        }

        if (segments[0] === 'planner') {
            const view = segments[1];
            const date = segments[2];

            if (view === 'daily') {
                if (!date) {
                    return this.redirect(`/planner/daily/${todayIso()}`);
                }
                return {
                    screen: new DailyPlannerScreen().init(date),
                    location: { kind: 'planner', view, date },
                };
            }
            if (view === 'weekly') {
                if (!date) {
                    return this.redirect(`/planner/weekly/${thisWeekIso()}`);
                }
                return {
                    screen: new WeeklyPlannerScreen().init(date),
                    location: { kind: 'planner', view, date },
                };
            }
            if (view === 'monthly') {
                if (!date) {
                    return this.redirect(`/planner/monthly/${thisMonthIso()}`);
                }
                return {
                    screen: new MonthlyPlannerScreen().init(date),
                    location: { kind: 'planner', view, date },
                };
            }
            if (view === 'annual') {
                if (!date) {
                    return this.redirect(`/planner/annual/${thisYear()}`);
                }
                return {
                    screen: new AnnualPlannerScreen().init(date),
                    location: { kind: 'planner', view, date },
                };
            }
        }

        if (segments[0] === 'types' && segments[1]) {
            const title = decodeURIComponent(segments[1]);
            return {
                screen: new TypeScreen().init(title),
                location: { kind: 'type', title },
            };
        }

        if (segments[0] === 'settings') {
            const s = new SettingsScreen();
            const section = (segments[1] ?? 'attributes') as SettingsSection;
            s.switchScreen(section);
            return {
                screen: s,
                location: { kind: 'settings', section },
            };
        }

        return this.notFound(path);
    }

    private redirect(path: string): RouteMatch {
        return {
            screen: document.createElement('div'),
            location: this.location,
            redirectTo: path,
        };
    }

    private notFound(path: string): RouteMatch {
        const div = document.createElement('div');
        div.textContent = `404 Not Found: ${path}`;
        return {
            screen: div,
            location: { kind: 'not_found', path },
        };
    }
}
