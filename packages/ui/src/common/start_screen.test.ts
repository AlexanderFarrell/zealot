import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AppSettings } from '@websoil/engine';
import { openStartScreen, rememberLastItem } from './start_screen';

const getById = vi.hoisted(() => vi.fn());
vi.mock('@zealot/api/src/item', () => ({ ItemAPI: class { GetById = getById; } }));

beforeEach(() => {
    localStorage.clear();
    getById.mockReset();
    AppSettings.set({ start_screen: 'home' });
});
afterEach(() => vi.restoreAllMocks());

describe('openStartScreen', () => {
    it('does nothing for Home', () => {
        const navigate = vi.fn();
        expect(openStartScreen('/', navigate)).toBe(false);
        expect(navigate).not.toHaveBeenCalled();
    });

    it('never redirects a deep link or in-app path', () => {
        AppSettings.set({ start_screen: 'daily' });
        const navigate = vi.fn();
        expect(openStartScreen('/item/Foo', navigate)).toBe(false);
        expect(navigate).not.toHaveBeenCalled();
    });

    it('opens the daily planner for today straight away', () => {
        AppSettings.set({ start_screen: 'daily' });
        const navigate = vi.fn();
        expect(openStartScreen('/', navigate)).toBe(true);
        expect(navigate).toHaveBeenCalledWith(expect.stringMatching(/^\/planner\/daily\/\d{4}-\d{2}-\d{2}$/));
    });

    it('opens the last item when it still exists', async () => {
        AppSettings.set({ start_screen: 'last_item' });
        rememberLastItem(42);
        getById.mockResolvedValue({});
        const navigate = vi.fn();
        expect(openStartScreen('/', navigate)).toBe(false);
        await vi.waitFor(() => expect(navigate).toHaveBeenCalledWith('/item_id/42'));
    });

    it('stays on Home when the last item is gone or none was saved', async () => {
        AppSettings.set({ start_screen: 'last_item' });
        const navigate = vi.fn();
        openStartScreen('/', navigate);
        rememberLastItem(7);
        getById.mockRejectedValue(new Error('404'));
        openStartScreen('/', navigate);
        await Promise.resolve();
        await Promise.resolve();
        expect(navigate).not.toHaveBeenCalled();
    });
});
