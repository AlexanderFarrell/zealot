interface MobileLoginOpts {
    onLogin: (username: string, password: string) => Promise<void>;
}

export class MobileLoginScreen extends HTMLElement {
    private opts: MobileLoginOpts | null = null;

    init(opts: MobileLoginOpts): this {
        this.opts = opts;
        this.render();
        return this;
    }

    private render(): void {
        this.className = 'modal_background auth_modal';
        this.innerHTML = `
        <form id="login_form" class="inner_window">
            <h1 style="text-align: center;">Sign In</h1>
            <label for="mob_username">Username</label>
            <input
                type="text"
                id="mob_username"
                name="username"
                autocomplete="username"
                autocorrect="off"
                autocapitalize="none"
                spellcheck="false"
            >
            <label for="mob_password">Password</label>
            <input
                type="password"
                id="mob_password"
                name="password"
                autocomplete="current-password"
            >
            <button type="submit">Sign In</button>
            <p id="login_error" class="error" style="display:none;"></p>
        </form>
        `;

        const form = this.querySelector<HTMLFormElement>('#login_form')!;
        const errorEl = this.querySelector<HTMLParagraphElement>('#login_error')!;
        const submitBtn = form.querySelector<HTMLButtonElement>('button[type="submit"]')!;

        form.addEventListener('submit', (e) => {
            e.preventDefault();
            void this.handleSubmit(form, errorEl, submitBtn);
        });
    }

    private async handleSubmit(
        form: HTMLFormElement,
        errorEl: HTMLParagraphElement,
        submitBtn: HTMLButtonElement,
    ): Promise<void> {
        const data = new FormData(form);
        const username = (data.get('username') as string).trim();
        const password = data.get('password') as string;

        errorEl.style.display = 'none';
        errorEl.textContent = '';
        submitBtn.disabled = true;
        submitBtn.textContent = 'Signing in…';

        try {
            await this.opts!.onLogin(username, password);
        } catch (err) {
            const msg = err instanceof Error ? err.message : 'Sign in failed.';
            errorEl.textContent = msg;
            errorEl.style.display = '';
            submitBtn.disabled = false;
            submitBtn.textContent = 'Sign In';
        }
    }

    showError(msg: string): void {
        const errorEl = this.querySelector<HTMLParagraphElement>('#login_error');
        if (errorEl) {
            errorEl.textContent = msg;
            errorEl.style.display = '';
        }
        const submitBtn = this.querySelector<HTMLButtonElement>('button[type="submit"]');
        if (submitBtn) {
            submitBtn.disabled = false;
            submitBtn.textContent = 'Sign In';
        }
    }
}

customElements.define('mobile-login-screen', MobileLoginScreen);
