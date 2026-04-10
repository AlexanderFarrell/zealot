import { BaseElementEmpty, NavigationCommands, commands } from "@websoil/engine";
import { ZealotIcon, icons } from "@zealot/content";

export class HeaderBar extends BaseElementEmpty {
    async render() {
        this.classList.add('box');
        this.innerHTML = `
        <button name="home">
            <img src="${ZealotIcon}" alt="Zealot">
            <span>Zealot</span>
        </button>
        <div class="header-bar-spacer"></div>
        <button name="hamburger" class="mobile_only">
            <img src="${icons.hamburger}" alt="Menu">
        </button>
        `;

        this.querySelector<HTMLButtonElement>('[name="home"]')!
            .addEventListener('click', () => commands.runner.run(NavigationCommands.goHome));

        this.querySelector<HTMLButtonElement>('[name="hamburger"]')!
            .addEventListener('click', () => {
                const bar = document.querySelector('mobile-title-bar') as (HTMLElement & { toggleDropdown(): void }) | null;
                bar?.toggleDropdown();
            });
    }
}

customElements.define('header-bar', HeaderBar);