import AuthAPI from "../api/auth";
import { events } from "../core/events";

let auth_modal: any | null = null ;

class AuthModal extends HTMLElement {
    connectedCallback() {
        this.classList.add("auth_modal")
        this.classList.add('modal_background')

        if (auth_modal != null) {
            this.remove();
        }

        let on_login: any;
        on_login = () => {
            this.remove();
            events.off('on_login', on_login);
        }
        events.on('on_login', on_login);
        this.to_login();
    }

    to_login () {
        this.innerHTML = `

        <form id="login" class="inner_window">
        <h1>Login</h1>
        <label for="username">Username:</label>
        <input type="text" id="username" name="username">

        <label for="password">Password:</label>
        <input type="password" id="password" name="password">

        <button type="submit">Login</button>
        <a id="register_instead">Register instead</a>
        <div id="error_message" class="error"></div>
        </form>
        `

        let form = this.querySelector('#login')! as HTMLFormElement;
        let error_message_view = this.querySelector("#error_message")! as HTMLDivElement;
        form.addEventListener('submit', async (e: SubmitEvent) => {
            e.preventDefault();
            const data = new FormData(form);
            try {
                await AuthAPI.login(data.get('username') as string, data.get('password') as string);
            } catch (e) {
                error_message_view.innerText = (e as Error).message; 
            }
        })
        let register_instead = this.querySelector('#register_instead')! as HTMLAnchorElement;
        register_instead.addEventListener('click', () => {
            this.to_register_screen();
        })
    }

    to_register_screen () {
        this.innerHTML = `

        <form id="register" class='inner_window'>
        <h1>Login</h1>
        <label for="username">Username:</label>
        <input type="text" id="username" name="username">

        <label for="password">Password:</label>
        <input type="password" id="password" name="password">

        <label for="confirm">Confirm:</label>
        <input type="confirm" id="confirm" name="confirm">

        <label for="email">Email:</label>
        <input type="email" id="email" name="email">

        <label for="name">Email:</label>
        <input type="text" id="name" name="name">

        <button type="submit">Login</button>
        <a id="register_instead">Register instead</a>
        <div id="error_message" class="error"></div>
        </form>
        `

        let form = this.querySelector("#register")! as HTMLFormElement;
        let error_message_view = this.querySelector("#error_message")! as HTMLDivElement;
        form.addEventListener('submit', async (e: SubmitEvent) => {
            e.preventDefault();
            const data = new FormData(form);
            try {
                await AuthAPI.register(data.get('username') as string,
                 data.get('password') as string,
                 data.get('confirm') as string,
                 data.get('name') as string,
                 data.get('email') as string
            );
            } catch (e) {
                error_message_view.innerText = (e as Error).message; 
            }
        })
    }
}

customElements.define('auth-modal', AuthModal)

export default AuthModal;