import ZealotAPI from "@zealot/api";
import { AuthAPI } from "@zealot/api/src/auth";
import { ItemAPI } from "@zealot/api/src/item";
import { InitCSRFEndpoint } from "@websoil/engine";
import { configureUserSettingsAPIs } from "@zealot/ui/src/screens/user_settings_screen";

InitCSRFEndpoint('/api/auth/');

const API = new ZealotAPI('/api');

configureUserSettingsAPIs(new AuthAPI('/api'), new ItemAPI('/api'));

export {
    API
}