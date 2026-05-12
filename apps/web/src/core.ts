import ZealotAPI from "@zealot/api";
import { InitCSRFEndpoint } from "@websoil/engine";

InitCSRFEndpoint('/api/auth/');

let API = new ZealotAPI('/api');

export {
    API
}