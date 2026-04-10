import { BaseElement, Events, Popups } from "@websoil/engine";
import { AuthAPI } from "@zealot/api/src/auth";
import { account } from "@zealot/domain";
import { AuthScreen } from "../auth/auth_screen";
import { LoadingSpinner } from "../common/loading_spinner";

interface MainScreenData {
    AuthAPI: AuthAPI,
    MainContentBuilder: () => HTMLElement
}

export class MainScreen extends BaseElement<MainScreenData> {
    public async render() {
        Events.on('to_auth', () => {
            this.to_auth();
        })

        Events.on('to_app', () => {
            this.to_app();
        })

        this.appendChild(new LoadingSpinner());
        try {
            const loggedIn = await this.data!.AuthAPI.IsLoggedIn();
            this.innerHTML = "";
            if (!loggedIn) {
                Events.emit('to_auth');
            } else {
                Events.emit('to_app');
            }
        } catch (e) {
            this.innerHTML = "";
            Popups.add_error((e as Error).message ?? 'Failed to check login status.');
            Events.emit('to_auth');
        }
    }

    private to_auth() {
        this.innerHTML = "";
        this.appendChild(new AuthScreen().init({
            on_login_attempt: async (dto: account.LoginBasicDto) => {
                try {
                    const account = await this.data!.AuthAPI.Basic.login(dto);
                    if (account) {
                        Events.emit('to_app');
                    }
                } catch (e) {
                    Popups.add_error((e as Error).message ?? 'Login failed.');
                }
            },
            on_register_attempt: async (dto: account.RegisterBasicDto) => {
                try {
                    const account = await this.data!.AuthAPI.Basic.register(dto);
                    if (account) {
                        Events.emit('to_app');
                    }
                } catch (e) {
                    Popups.add_error((e as Error).message ?? 'Registration failed.');
                }
            }
        }));
    }

    private to_app() {
        this.innerHTML = "";
        this.appendChild(this.data!.MainContentBuilder());
    }
}

customElements.define('zealot-main-screen', MainScreen);

// async function Main() {
//     if (!await API.Auth.IsLoggedIn()) {
//         Events.emit('to_auth');
//     } else {
//         Events.emit('to_app');
//     }
// }

// Events.on('to_auth', () => {
//     let main = GetMainDiv();
//     main.innerHTML = "";
//     main.appendChild(new AuthScreen().init({
//         on_login_attempt: async (dto: account.LoginBasicDto) => {
//             let account = await API.Auth.Basic.login(dto);
//             if (account) {
//                 Events.emit('to_app');
//             }
//         },
//         on_register_attempt: async (dto: account.RegisterBasicDto) => {
//             let account = await API.Auth.Basic.register(dto);
//             if (account) {
//                 Events.emit('to_app');
//             }
//         }
//     }))
// })

// Events.on('to_app', () => {
//     let main = GetMainDiv();
//     main.innerHTML = "";
//     main.appendChild(new ZealotWebClient());
// })
