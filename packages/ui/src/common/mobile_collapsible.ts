/**
 * Make a section's heading a tap-to-reveal toggle on phones, starting collapsed.
 * Desktop always shows the section and the heading stays plain text (see common.scss).
 */
export function makeMobileCollapsible(section: HTMLElement, heading: HTMLElement, title: string): void {
    section.classList.add('mobile-collapsible', 'mobile-collapsible--collapsed');
    const toggle = document.createElement('button');
    toggle.type = 'button';
    toggle.className = 'mobile-collapsible-toggle';
    toggle.setAttribute('aria-expanded', 'false');
    toggle.textContent = title;
    toggle.addEventListener('click', () => {
        const collapsed = section.classList.toggle('mobile-collapsible--collapsed');
        toggle.setAttribute('aria-expanded', String(!collapsed));
    });
    heading.replaceChildren(toggle);
}
