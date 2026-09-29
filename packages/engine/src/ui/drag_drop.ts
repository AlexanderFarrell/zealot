export interface DropZone {
    element: HTMLElement;
    onDragOver: (e: DragEvent) => void;
    onDrop: (draggedItemId: number) => void;
    onDragLeave?: () => void;
}

const zones = new Map<HTMLElement, DropZone>();

export function registerDropZone(zone: DropZone): void {
    zones.set(zone.element, zone);
}

export function unregisterDropZone(element: HTMLElement): void {
    zones.delete(element);
}

export function unregisterDropZonesIn(container: HTMLElement): void {
    for (const [el] of zones) {
        if (container.contains(el)) {
            zones.delete(el);
        }
    }
}

function getDraggedItemId(e: DragEvent): number | null {
    const raw = e.dataTransfer?.getData('text/plain');
    if (!raw) return null;
    const id = Number(raw);
    return Number.isFinite(id) && id > 0 ? id : null;
}

function findZoneForTarget(target: EventTarget | null): DropZone | null {
    if (!(target instanceof Node)) return null;
    // Walk up the zone list looking for the deepest containing element
    let best: DropZone | null = null;
    for (const [el, zone] of zones) {
        if (el.contains(target as Node)) {
            if (!best || el.contains(best.element)) {
                best = zone;
            }
        }
    }
    return best;
}

let currentZone: DropZone | null = null;

if (typeof document !== 'undefined') {
    document.addEventListener('dragover', (e: DragEvent) => {
        const zone = findZoneForTarget(e.target);
        if (!zone) {
            if (currentZone) {
                currentZone.onDragLeave?.();
                currentZone = null;
            }
            return;
        }

        if (currentZone && currentZone !== zone) {
            currentZone.onDragLeave?.();
        }
        currentZone = zone;
        e.preventDefault();
        zone.onDragOver(e);
    });

    document.addEventListener('dragleave', (e: DragEvent) => {
        // Only fire when leaving the document or moving to a non-zone area
        const related = e.relatedTarget;
        if (related === null) {
            // Left the browser window
            currentZone?.onDragLeave?.();
            currentZone = null;
            return;
        }

        const zone = findZoneForTarget(e.target);
        if (!zone) return;

        // Check if we're moving into a child of the same zone element
        if (zone.element.contains(related as Node)) return;

        // Moved out of this zone
        if (currentZone === zone) {
            zone.onDragLeave?.();
            currentZone = null;
        }
    });

    document.addEventListener('drop', (e: DragEvent) => {
        const zone = findZoneForTarget(e.target);
        if (!zone) return;

        e.preventDefault();
        zone.onDragLeave?.();
        currentZone = null;

        const id = getDraggedItemId(e);
        if (id !== null) {
            zone.onDrop(id);
        }
    });

    document.addEventListener('dragend', () => {
        currentZone?.onDragLeave?.();
        currentZone = null;
    });
}
