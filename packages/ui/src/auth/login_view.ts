import {BaseElement, BaseElementEmpty} from '@websoil/engine';
import { ZealotIcon } from '@zealot/content';
import { account } from '@zealot/domain';

export interface LoginViewInfo {
    on_login_attempt: (dto: account.LoginBasicDto) => Promise<void>;
}

export class LoginView extends BaseElement<LoginViewInfo> {
    render() {
        this.innerHTML = `
        <form id="login" class="inner_window">
            <img src="${ZealotIcon}" style="padding: 0 30%">
            <h1 style="text-align: center;">Login to Zealot</h1>

            <label for="username">Username:</label>
            <input type="text" id="username" name="username" autocomplete="username">

            <label for="password">Password:</label>
            <input type="password" id="password" name="password" autocomplete="current-password">

            <button type="submit" style="margin-top: 0.25em">Login</button>
            <a id="register_instead">Create Account Instead</a>
            <div id="error_message" class="error"></div>
        </form>
        `

        let form = this.querySelector('#login')! as HTMLFormElement;
        let error_msg_view = this.querySelector('#error_message')! as HTMLDivElement;
        let back_button = this.querySelector('#register_instead')! as HTMLAnchorElement;

        form.addEventListener('submit', async (e: SubmitEvent) => {
            e.preventDefault();
            const data = new FormData(form);
            const dto: account.LoginBasicDto = {
                username: data.get('username') as string,
                password: data.get('password') as string
            };
            try {
                await this.data!.on_login_attempt(dto);
            } catch (e) {
                error_msg_view.innerText = (e as Error).message;
            }
        })

        back_button.addEventListener('click', () => {
            this.dispatchEvent(new Event('back', {bubbles: true}));
        })
    }
}

customElements.define('login-screen', LoginView);
