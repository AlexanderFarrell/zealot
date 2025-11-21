import AuthAPI from "../api/auth";
import AddItemModal from "./add_item_modal";

class AuthModal extends HTMLElement {
    connectedCallback() {
        this.classList.add("auth_modal")
    }

    to_login () {
        this.innerHTML = `
        <h1>Login</h1>

        <form id="login">
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

    }
}

customElements.define('add-item-modal', AddItemModal)