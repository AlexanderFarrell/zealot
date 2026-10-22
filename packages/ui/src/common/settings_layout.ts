/** Building blocks shared by every Settings tab, so they look and behave alike. */

export interface SectionHeader {
    element: HTMLElement;
    /** The "+ Add ..." button, present when the section has an add form. */
    addButton: HTMLButtonElement | null;
}

/** A title and description, with an optional primary action on the right. */
export function buildSectionHeader(title: string, description: string, addLabel?: string): SectionHeader {
    const element = document.createElement('div');
    element.className = 'settings-header';

    const text = document.createElement('div');
    const heading = document.createElement('h2');
    heading.textContent = title;
    const copy = document.createElement('p');
    copy.className = 'tool-muted';
    copy.textContent = description;
    text.append(heading, copy);
    element.appendChild(text);

    let addButton: HTMLButtonElement | null = null;
    if (addLabel) {
        addButton = document.createElement('button');
        addButton.type = 'button';
        addButton.className = 'settings-add';
        addButton.textContent = addLabel;
        element.appendChild(addButton);
    }
    return { element, addButton };
}

/**
 * Wraps an add form in a panel that the section's Add button opens and closes. Existing entries
 * stay on top; the form is out of the way until it is asked for.
 */
export function buildAddPanel(form: HTMLElement, addButton: HTMLButtonElement, addLabel: string, open: boolean): HTMLElement {
    const panel = document.createElement('div');
    panel.className = 'settings-add-panel';
    panel.appendChild(form);

    const set = (visible: boolean) => {
        panel.hidden = !visible;
        addButton.textContent = visible ? 'Cancel' : addLabel;
        addButton.setAttribute('aria-expanded', String(visible));
        if (visible) form.querySelector<HTMLElement>('input, select, textarea')?.focus();
    };
    set(open);
    addButton.addEventListener('click', () => set(panel.hidden));
    return panel;
}

/** A collapsed-by-default group, for material that is rarely needed (system kinds, and so on). */
export function buildDisclosure(summary: string, ...children: HTMLElement[]): HTMLDetailsElement {
    const details = document.createElement('details');
    details.className = 'settings-disclosure';
    const label = document.createElement('summary');
    label.textContent = summary;
    details.append(label, ...children);
    return details;
}

/** What a section shows when there is nothing to list or configure yet. */
export function buildEmptyState(title: string, description: string): HTMLElement {
    const empty = document.createElement('div');
    empty.className = 'settings-empty';
    const heading = document.createElement('h3');
    heading.textContent = title;
    const copy = document.createElement('p');
    copy.className = 'tool-muted';
    copy.textContent = description;
    empty.append(heading, copy);
    return empty;
}
