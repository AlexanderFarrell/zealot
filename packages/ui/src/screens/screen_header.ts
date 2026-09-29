export interface ScreenAction {
    iconURL?: string;
    label: string;
    onClick: () => void;
}

export function createScreenHeader(
    title: string,
    actions?: ScreenAction[],
): HTMLElement {
    const header = document.createElement('header');
    header.className = 'screen-header';

    const h1 = document.createElement('h1');
    h1.className = 'screen-header-title';
    h1.textContent = title;
    header.appendChild(h1);

    if (actions && actions.length > 0) {
        const bar = document.createElement('div');
        bar.className = 'screen-header-actions';

        for (const action of actions) {
            const btn = document.createElement('button');
            btn.type = 'button';
            btn.className = 'screen-header-btn';
            btn.addEventListener('click', action.onClick);

            if (action.iconURL) {
                const img = document.createElement('img');
                img.src = action.iconURL;
                img.alt = '';
                btn.appendChild(img);
            }

            const label = document.createElement('span');
            label.textContent = action.label;
            btn.appendChild(label);

            bar.appendChild(btn);
        }

        header.appendChild(bar);
    }

    return header;
}
