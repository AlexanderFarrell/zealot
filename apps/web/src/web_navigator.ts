import type { Navigator, PlannerView, SettingsSection } from '@websoil/engine';
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
    return new Date().toISOString().slice(0, 10);
}

function thisWeekIso(): string {
    const now = new Date();
    const year = now.getFullYear();
    // ISO week number
    const jan4 = new Date(year, 0, 4);
    const startOfWeek1 = new Date(jan4);
    startOfWeek1.setDate(jan4.getDate() - ((jan4.getDay() + 6) % 7));
    const diff = now.getTime() - startOfWeek1.getTime();
    const week = Math.floor(diff / (7 * 24 * 60 * 60 * 1000)) + 1;
    return `${year}-W${String(week).padStart(2, '0')}`;
}

function thisMonthIso(): string {
    const now = new Date();
    return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`;
}

export class WebNavigator implements Navigator {
    constructor() {
        window.addEventListener('popstate', () => this.renderCurrent());
    }

    private getContent(): HTMLElement {
        const el = document.querySelector('center-content');
        if (!el) throw new Error('No <center-content> element found');
        return el as HTMLElement;
    }

    private push(path: string): void {
        history.pushState(null, '', path);
        this.renderCurrent();
    }

    openHome(): void {
        this.push('/');
    }

    openItem(title: string): void {
        this.push(`/item/${encodeURIComponent(title)}`);
    }

    openItemById(id: number): void {
        this.push(`/item_id/${id}`);
    }

    openMedia(path: string): void {
        this.push(`/media/${path}`);
    }

    openPlanner(view: PlannerView, date?: string): void {
        switch (view) {
            case 'daily':
                this.push(`/planner/daily/${date ?? todayIso()}`);
                break;
            case 'weekly':
                this.push(`/planner/weekly/${date ?? thisWeekIso()}`);
                break;
            case 'monthly':
                this.push(`/planner/monthly/${date ?? thisMonthIso()}`);
                break;
            case 'annual':
                this.push(`/planner/annual/${date ?? String(new Date().getFullYear())}`);
                break;
        }
    }

    openTypes(): void {
        this.push('/types');
    }

    openType(title: string): void {
        this.push(`/types/${encodeURIComponent(title)}`);
    }

    openAnalysis(): void {
        this.push('/analysis');
    }

    openRules(): void {
        this.push('/rules');
    }

    openSettings(section?: SettingsSection): void {
        this.push(`/settings/${section ?? 'attributes'}`);
    }

    resolve(): void {
        this.renderCurrent();
    }

    private renderCurrent(): void {
        const path = window.location.pathname;
        const content = this.getContent();
        const screen = this.matchRoute(path);
        content.innerHTML = '';
        content.appendChild(screen);
    }

    private matchRoute(path: string): HTMLElement {
        // Exact matches
        if (path === '/') {
            const s = new ItemScreen();
            s.loadItem('Home');
            return s;
        }
        if (path === '/analysis') {
            return new AnalysisScreen();
        }
        if (path === '/rules') {
            return new RulesScreen();
        }
        if (path === '/types') {
            return new TypesScreen();
        }

        // Prefix matches
        const segments = path.split('/').filter(Boolean);

        if (segments[0] === 'item' && segments[1]) {
            const s = new ItemScreen();
            s.loadItem(decodeURIComponent(segments[1]));
            return s;
        }

        if (segments[0] === 'item_id' && segments[1]) {
            const s = new ItemScreen();
            s.loadItemById(parseInt(segments[1], 10));
            return s;
        }

        if (segments[0] === 'media') {
            // Rejoin the rest of the path after /media/
            const mediaPath = path.replace(/^\/media\//, '');
            return new MediaScreen().init(mediaPath);
        }

        if (segments[0] === 'planner') {
            const view = segments[1];
            const date = segments[2];

            if (view === 'daily') {
                if (!date) { this.openPlanner('daily'); return document.createElement('div'); }
                return new DailyPlannerScreen().init(date);
            }
            if (view === 'weekly') {
                if (!date) { this.openPlanner('weekly'); return document.createElement('div'); }
                return new WeeklyPlannerScreen().init(date);
            }
            if (view === 'monthly') {
                if (!date) { this.openPlanner('monthly'); return document.createElement('div'); }
                return new MonthlyPlannerScreen().init(date);
            }
            if (view === 'annual') {
                if (!date) { this.openPlanner('annual'); return document.createElement('div'); }
                return new AnnualPlannerScreen().init(date);
            }
        }

        if (segments[0] === 'types' && segments[1]) {
            return new TypeScreen().init(decodeURIComponent(segments[1]));
        }

        if (segments[0] === 'settings') {
            const s = new SettingsScreen();
            s.switchScreen(segments[1] ?? 'attributes');
            return s;
        }

        // 404
        const div = document.createElement('div');
        div.textContent = `404 Not Found: ${path}`;
        return div;
    }
}
