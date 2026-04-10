import { API } from "./core";
import ZealotWebClient from "./web_client";
import { MainScreen } from "@zealot/ui/src/screens/main_screen";
import { Popups } from "@websoil/engine";

window.addEventListener('unhandledrejection', (e) => {
    console.error('Unhandled promise rejection:', e.reason);
    Popups.add_error(e.reason?.message ?? 'An unexpected error occurred.');
});
window.addEventListener('error', (e) => {
    console.error('Unhandled error:', e.error);
});

let main = document.querySelector<HTMLDivElement>('#app')!;
main.innerHTML = "";
main.appendChild(new MainScreen().init({
    MainContentBuilder: () => {
        return new ZealotWebClient()
    },
    AuthAPI: API.Auth
}))
