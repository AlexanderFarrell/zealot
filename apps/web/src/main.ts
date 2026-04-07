import { API } from "./core";
import ZealotWebClient from "./web_client";
import { MainScreen } from "@zealot/ui/src/screens/main_screen";

let main = document.querySelector<HTMLDivElement>('#app')!;
main.innerHTML = "";
main.appendChild(new MainScreen().init({
    MainContentBuilder: () => {
        return new ZealotWebClient()
    },
    AuthAPI: API.Auth
}))
