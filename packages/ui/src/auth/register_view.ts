import {BaseElement, BaseElementEmpty} from '@websoil/engine';
import { account } from '@zealot/domain';

export interface RegisterViewInfo {
    on_register_attempt: (dto: account.RegisterBasicDto) => Promise<void>;
}

export class RegisterView extends BaseElement<RegisterViewInfo> {
    render() {
        this.innerHTML = `
        <form id="register" class='inner_window'>
            <h1>Register New Account</h1>
            <label for="username">Username:</label>
            <input type="text" id="username" name="username" autocomplete="username">

            <label for="password">Password:</label>
            <input type="password" id="password" name="password" autocomplete="new-password">

            <label for="confirm">Confirm:</label>
            <input type="password" id="confirm" name="confirm">

            <label for="email">Email:</label>
            <input type="email" id="email" name="email" autocomplete="email">

            <label for="name">Given Name:</label>
            <input type="text" id="given_name" name="given_name" autocomplete="given-name">

            <label for="name">Surname</label>
            <input type="text" id="surname" name="surname" autocomplete="family-name">

            <button type="submit">Register</button>
            <a id="login_instead">Already have an account? Login</a>
            <div id="error_message" class="error"></div>
        </form>
        `

        let form = this.querySelector('#register')! as HTMLFormElement;
        let error_msg_view = this.querySelector('#error_message')! as HTMLDivElement;
        let back_button = this.querySelector('#login_instead')! as HTMLAnchorElement;

        form.addEventListener('submit', async (e: SubmitEvent) => {
            e.preventDefault();
            const data = new FormData(form);
            const dto: account.RegisterBasicDto = {
                username: data.get('username') as string,
                password: data.get('password') as string,
                confirm: data.get('confirm') as string,
                email: data.get('email') as string,
                given_name: data.get('given_name') as string,
                surname: data.get('surname') as string
            };
            try {
                await this.data!.on_register_attempt(dto);
            } catch (e) {
                error_msg_view.innerText = (e as Error).message;
            }
        })

        back_button.addEventListener('click', () => {
            this.dispatchEvent(new Event('back', {bubbles: true}));
        })
    }
}

customElements.define('register-screen', RegisterView);