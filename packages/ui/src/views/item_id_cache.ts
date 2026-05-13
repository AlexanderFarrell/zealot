/**
 * Shared cache for resolving item IDs to Item objects.
 *
 * Multiple components (ItemPickerInput, ItemChipsInput, etc.) may all try to
 * resolve the same item ID when an item screen renders. This module deduplicates
 * those fetches so each ID results in at most one in-flight network request.
 *
 * The cache is module-scoped (singleton per page load) and never evicts. This
 * is fine for item title display: stale titles are a minor UX issue and titles
 * are refreshed on full page reloads.
 */

import { ItemAPI } from '@zealot/api/src/item';
import type { Item } from '@zealot/domain/src/item';

const api = new ItemAPI('/api');
const cache = new Map<number, Promise<Item>>();

/**
 * Returns a promise that resolves to the Item with the given ID.
 * If the ID has already been requested (even if still in-flight), the same
 * promise is returned — no duplicate network call is made.
 *
 * On failure the cached promise is removed so the next caller retries.
 */
export function getCachedItem(id: number): Promise<Item> {
    if (!cache.has(id)) {
        const promise = api.GetById(id).catch((err: unknown) => {
            cache.delete(id);
            return Promise.reject(err);
        });
        cache.set(id, promise);
    }
    return cache.get(id)!;
}
