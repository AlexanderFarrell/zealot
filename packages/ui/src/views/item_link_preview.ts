import { ItemAPI } from '@zealot/api/src/item';
import type { Item } from '@zealot/domain/src/item';
import { renderItemTitle } from './item_title';
import { getCachedItem } from './item_id_cache';

const api = new ItemAPI('/api');
const titleCache = new Map<string, Promise<Item>>();

function getCachedItemByTitle(title: string): Promise<Item> {
    if (!titleCache.has(title)) {
        const promise = api.GetByTitle(title).catch((err: unknown) => {
            titleCache.delete(title);
            return Promise.reject(err);
        });
        titleCache.set(title, promise);
    }
    return titleCache.get(title)!;
}

function getSnippet(content: string): string {
    const plain = content
        .replace(/^\s*#{1,6}\s+/gm, '')
        .replace(/\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/g, (_, target, label: string | undefined) => label ?? target)
        .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
        .replace(/[*_`~]/g, '')
        .replace(/\n+/g, ' ')
        .trim();
    return plain.length > 120 ? plain.slice(0, 120) + '…' : plain;
}

function positionPanel(panel: HTMLElement, anchor: HTMLElement): void {
    const rect = anchor.getBoundingClientRect();
    const GAP = 6;
    const panelW = 360;
    const panelH = 120;

    let top = rect.bottom + GAP;
    let left = rect.left;

    if (left + panelW > window.innerWidth - 8) {
        left = Math.max(8, window.innerWidth - panelW - 8);
    }
    if (top + panelH > window.innerHeight - 8) {
        top = rect.top - panelH - GAP;
    }

    panel.style.top = `${top}px`;
    panel.style.left = `${left}px`;
}

export function initLinkPreview(): void {
    const panel = document.createElement('div');
    panel.id = 'zealot-link-preview';
    panel.className = 'zealot-link-preview';
    panel.hidden = true;

    const titleEl = document.createElement('div');
    titleEl.className = 'zealot-link-preview__title';

    const snippetEl = document.createElement('div');
    snippetEl.className = 'zealot-link-preview__snippet';

    panel.appendChild(titleEl);
    panel.appendChild(snippetEl);
    document.body.appendChild(panel);

    let showTimer: ReturnType<typeof setTimeout> | null = null;
    let currentAnchor: HTMLElement | null = null;

    function hide(): void {
        if (showTimer !== null) {
            clearTimeout(showTimer);
            showTimer = null;
        }
        panel.hidden = true;
        currentAnchor = null;
    }

    document.addEventListener('mouseover', (e) => {
        const anchor = (e.target as HTMLElement).closest<HTMLAnchorElement>('a[href]');
        const href = anchor?.getAttribute('href') ?? '';

        if (!anchor || !href.startsWith('zealot://item/')) {
            // Moved off an item link — hide unless still inside the current anchor
            if (currentAnchor && !currentAnchor.contains(e.target as Node)) {
                hide();
            }
            return;
        }

        if (currentAnchor === anchor) return;

        // New item link — cancel any pending show and start fresh
        if (showTimer !== null) clearTimeout(showTimer);
        currentAnchor = anchor;
        panel.hidden = true;

        showTimer = setTimeout(() => {
            showTimer = null;
            if (currentAnchor !== anchor) return;

            const raw = decodeURIComponent(href.slice('zealot://item/'.length));
            const hashIdx = raw.indexOf('#');
            const target = hashIdx >= 0 ? raw.slice(0, hashIdx) : raw;

            const numericId = Number(target);
            const fetch = Number.isFinite(numericId) && numericId > 0
                ? getCachedItem(numericId)
                : getCachedItemByTitle(target);

            fetch.then((item) => {
                if (currentAnchor !== anchor) return;
                titleEl.replaceChildren();
                renderItemTitle(titleEl, item);
                snippetEl.textContent = getSnippet(item.Content);
                positionPanel(panel, anchor);
                panel.hidden = false;
            }).catch(() => { /* no preview for missing items */ });
        }, 200);
    });

    document.addEventListener('mouseleave', () => hide(), true);
}
