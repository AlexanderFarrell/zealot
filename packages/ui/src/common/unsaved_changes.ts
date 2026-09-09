export type LeavePrompt = {
    title: string;
    message: string;
    discardLabel: string;
    saveLabel?: string;
};

export type PendingWork = {
    /** A stable, human-readable identity used only for diagnostics. */
    id: string;
    /** Returns true when leaving would discard user work. */
    isDirty: () => boolean;
    /** Flushes recoverable autosaves before navigation. Return false on failure. */
    flush?: () => Promise<boolean>;
    /** Copy for an explicit, non-autosaved draft. */
    prompt: () => LeavePrompt;
    /** Clears an explicitly discarded draft before leaving. */
    discard?: () => void;
};

type LeaveDecision = 'stay' | 'discard' | 'save';

const pendingWork = new Map<string, PendingWork>();
let dialogOpen = false;

export function registerPendingWork(work: PendingWork): () => void {
    pendingWork.set(work.id, work);
    return () => pendingWork.delete(work.id);
}

export function hasPendingWork(): boolean {
    return [...pendingWork.values()].some((work) => work.isDirty());
}

function showLeaveDialog(prompt: LeavePrompt): Promise<LeaveDecision> {
    return new Promise((resolve) => {
        const previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
        const dialog = document.createElement('div');
        dialog.className = 'unsaved-changes-dialog modal_background';
        dialog.innerHTML = `
            <div class="inner_window unsaved-changes-dialog__window" role="dialog" aria-modal="true" aria-labelledby="unsaved-changes-title">
                <h2 id="unsaved-changes-title"></h2>
                <p class="unsaved-changes-dialog__message"></p>
                <div class="unsaved-changes-dialog__actions">
                    <button type="button" data-role="stay">Keep editing</button>
                    ${prompt.saveLabel ? '<button type="button" data-role="save"></button>' : ''}
                    <button type="button" class="danger" data-role="discard"></button>
                </div>
            </div>
        `;

        dialog.querySelector('h2')!.textContent = prompt.title;
        dialog.querySelector('.unsaved-changes-dialog__message')!.textContent = prompt.message;
        dialog.querySelector<HTMLButtonElement>('[data-role="save"]')?.replaceChildren(prompt.saveLabel ?? 'Save and leave');
        dialog.querySelector<HTMLButtonElement>('[data-role="discard"]')!.textContent = prompt.discardLabel;

        const close = (decision: LeaveDecision) => {
            dialog.remove();
            if (previousFocus?.isConnected) previousFocus.focus();
            resolve(decision);
        };
        dialog.querySelector('[data-role="stay"]')!.addEventListener('click', () => close('stay'));
        dialog.querySelector('[data-role="save"]')?.addEventListener('click', () => close('save'));
        dialog.querySelector('[data-role="discard"]')!.addEventListener('click', () => close('discard'));
        dialog.addEventListener('click', (event) => {
            if (event.target === dialog) close('stay');
        });
        dialog.addEventListener('keydown', (event) => {
            if (event.key === 'Escape') {
                event.preventDefault();
                close('stay');
            }
        });
        (document.body ?? document.documentElement).appendChild(dialog);
        dialog.querySelector<HTMLButtonElement>('[data-role="stay"]')!.focus();
    });
}

/**
 * Runs the single leave decision used by navigators and local dismiss buttons.
 * Autosaves flush first; explicit drafts are never submitted automatically.
 */
export async function confirmLeave(works: Iterable<PendingWork> = pendingWork.values()): Promise<boolean> {
    if (dialogOpen) return false;
    const entries = [...works].filter((work) => work.isDirty());
    if (entries.length === 0) return true;

    dialogOpen = true;
    try {
        for (const work of entries) {
            if (!work.flush) continue;
            const saved = await work.flush();
            if (!saved && work.isDirty()) {
                const decision = await showLeaveDialog({
                    ...work.prompt(),
                    title: 'Changes could not be saved',
                    message: 'Your latest changes are still only on this device.',
                    discardLabel: 'Leave without saving',
                });
                if (decision === 'stay') return false;
                work.discard?.();
            }
        }

        for (const work of entries) {
            if (!work.isDirty() || work.flush) continue;
            const decision = await showLeaveDialog(work.prompt());
            if (decision === 'stay') return false;
            if (decision === 'save') return false;
            work.discard?.();
        }
        return true;
    } finally {
        dialogOpen = false;
    }
}

export async function confirmDiscard(work: PendingWork): Promise<boolean> {
    return confirmLeave([work]);
}

window.addEventListener('beforeunload', (event) => {
    if (!hasPendingWork()) return;
    event.preventDefault();
    event.returnValue = '';
});
