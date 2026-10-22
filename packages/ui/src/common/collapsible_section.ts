/**
 * Make a section's heading a click-to-reveal toggle, starting collapsed. Used for the Comments
 * section on items and planners, which stays out of the way until it is wanted (see common.scss).
 */
export function makeCollapsible(section: HTMLElement, heading: HTMLElement, title: string): void {
    section.classList.add('collapsible', 'collapsible--collapsed');
    const toggle = document.createElement('button');
    toggle.type = 'button';
    toggle.className = 'collapsible-toggle';
    toggle.setAttribute('aria-expanded', 'false');
    toggle.textContent = title;
    toggle.addEventListener('click', () => {
        const collapsed = section.classList.toggle('collapsible--collapsed');
        toggle.setAttribute('aria-expanded', String(!collapsed));
    });
    heading.replaceChildren(toggle);
}
