import {BaseElement} from '@websoil/engine';
import { account } from '@zealot/domain';
import {LoginView} from './login_view';
import { RegisterView } from './register_view';

export interface AuthScreenInfo {
    on_login_attempt: (dto: account.LoginBasicDto) => Promise<void>;
    on_register_attempt: (dto: account.RegisterBasicDto) => Promise<void>;
}

export class AuthScreen extends BaseElement<AuthScreenInfo> {
    render() {
        // Make this display a modal in the middle
        this.classList.add('modal_background');
        this.classList.add('auth_modal');

        // Start on login screen
        this.to_login();
    }

    to_login() {
        this.innerHTML = "";
        let view = new LoginView();
        view.init({on_login_attempt: this.data!.on_login_attempt});
        view.addEventListener('back', () => {
            this.to_register();
        })
        this.appendChild(view)
    }

    to_register() {
        this.innerHTML = "";
        let view = new RegisterView();
        view.init({on_register_attempt: this.data!.on_register_attempt});
        view.addEventListener('back', () => {
            this.to_login();
        })
        this.appendChild(view);
    }
}

customElements.define('auth-screen', AuthScreen);