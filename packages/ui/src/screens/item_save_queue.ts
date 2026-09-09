import { ItemAPI } from '@zealot/api/src/item';

type ItemPatch = { title?: string; content?: string };
type SaveStatus = 'saved' | 'saving' | 'failed';

const itemApi = new ItemAPI('/api');
const queues = new Map<number, ItemSaveQueue>();

export class ItemSaveQueue {
    private _queued: ItemPatch = {};
    private _timer: ReturnType<typeof setTimeout> | null = null;
    private _draining: Promise<boolean> | null = null;
    private _status: SaveStatus = 'saved';
    private readonly _listeners = new Set<(status: SaveStatus) => void>();

    constructor(private readonly itemId: number) {}

    get dirty(): boolean {
        return this._status !== 'saved' || Object.keys(this._queued).length > 0;
    }

    enqueue(patch: ItemPatch, delayMs = 700): void {
        this._queued = { ...this._queued, ...patch };
        if (this._timer) clearTimeout(this._timer);
        this._setStatus('saving');
        this._timer = setTimeout(() => {
            this._timer = null;
            void this.flush();
        }, delayMs);
    }

    subscribe(listener: (status: SaveStatus) => void): () => void {
        this._listeners.add(listener);
        listener(this._status);
        return () => this._listeners.delete(listener);
    }

    async flush(): Promise<boolean> {
        if (this._timer) {
            clearTimeout(this._timer);
            this._timer = null;
        }
        if (this._draining) return this._draining;
        this._draining = this._drain();
        try {
            return await this._draining;
        } finally {
            this._draining = null;
        }
    }

    private async _drain(): Promise<boolean> {
        while (Object.keys(this._queued).length > 0) {
            const patch = this._queued;
            this._queued = {};
            this._setStatus('saving');
            try {
                await itemApi.Update(this.itemId, { item_id: this.itemId, ...patch });
            } catch {
                this._queued = { ...patch, ...this._queued };
                this._setStatus('failed');
                return false;
            }
        }
        this._setStatus('saved');
        return true;
    }

    private _setStatus(status: SaveStatus): void {
        if (this._status === status) return;
        this._status = status;
        this._listeners.forEach((listener) => listener(status));
    }
}

export function getItemSaveQueue(itemId: number): ItemSaveQueue {
    let queue = queues.get(itemId);
    if (!queue) {
        queue = new ItemSaveQueue(itemId);
        queues.set(itemId, queue);
    }
    return queue;
}
