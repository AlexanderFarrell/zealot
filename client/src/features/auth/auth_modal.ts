import AuthAPI from "../../api/auth";
import { events } from "../../shared/events";
import ZealotApp from "../../app/zealot_app";
import ZealotIcon from "./../../../public/zealot.webp";

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
            this.to_zealot_app();
            events.off('on_login', on_login);
        }
        events.on('on_login', on_login);
        this.to_login();
    }

    to_login () {
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
        });
        (document.querySelector('#username')! as HTMLInputElement)?.focus();
    }

    to_register_screen () {
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

        <label for="name">Name:</label>
        <input type="text" id="name" name="name" autocomplete="given-name">

        <button type="submit">Register</button>
        <a id="login_instead">Already have an account? Login</a>
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
        });
        let login_button = this.querySelector('#login_instead')! as HTMLAnchorElement;
        login_button.addEventListener('click', () => {
            this.to_login();
        });
    }

    to_zealot_app() {
        document.body.innerHTML = "";
        document.body.appendChild(new ZealotApp());
    }
}

customElements.define('auth-modal', AuthModal)

export default AuthModal;