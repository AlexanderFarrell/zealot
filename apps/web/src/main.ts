import { Stylesheet } from "@zealot/content";
import { API } from "./core";
import { Events } from "@websoil/engine";
import { AuthScreen } from "@zealot/ui/src/auth/auth_screen";
import { account } from "@zealot/domain";
import ZealotWebClient from "./web_client";

function GetMainDiv(): HTMLDivElement {
    return document.querySelector<HTMLDivElement>('#app')!;
}

async function Main() {
    if (!await API.Auth.IsLoggedIn()) {
        Events.emit('to_auth');
    } else {
        Events.emit('to_app');
    }
}

Events.on('to_auth', () => {
    let main = GetMainDiv();
    main.innerHTML = "";
    main.appendChild(new AuthScreen().init({
        on_login_attempt: async (dto: account.LoginBasicDto) => {
            API.Auth.Basic.login(dto);
        },
        on_register_attempt: async (dto: account.RegisterBasicDto) => {
            API.Auth.Basic.register(dto);
        }
    }))
})

Events.on('to_app', () => {
    let main = GetMainDiv();
    main.innerHTML = "";
    main.appendChild(new ZealotWebClient());
})

Main();