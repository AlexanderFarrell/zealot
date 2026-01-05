import { events } from "../core/events";
import { router } from "../core/router";
import { get_req, get_json, patch_req, post_req } from "../core/api_helper";
import { get_settings, set_settings } from "../core/settings";

type User = {
    username: string;
    email: string;
    name: string;
    user_id: number;
}

export const AuthAPI = {
    current_user: null as User | null,

    login: async (username: string, password: string) => {
        const response = await post_req('/api/account/login', {
            username,
            password
        });
        await AuthAPI.handle_response(response);
    },

    is_logged_in: async () => {
        const response = await get_req('/api/account/is_logged_in');
        return response.ok || response.status == 201;
    },

    register: async (username: string, password: string, 
        confirm: string, name: string, email: string) => {
        const response = await post_req('/api/account/register', {
            username,
            password,
            confirm,
            name,
            email
        });
        events.emit('on_register_account');
        await AuthAPI.handle_response(response);
        router.navigate('/')
    },

    logout: async () => {
        // Let the server know, whether successful or not.
        await get_req('/api/account/logout');
        events.emit('on_logout');
        AuthAPI.require_relogin();
    },

    sync_settings: async () => {
        await patch_req('/api/account/settings', get_settings())
    },

    get_user_details: async () => {
        let data: any = await get_json('/api/account/details');
        AuthAPI.current_user = {
            username: data['username'],
            email: data['email'],
            name: data['email'],
            user_id: data['user_id']
        };
        set_settings(data['settings']);
    },

    require_relogin: () => {
        AuthAPI.current_user = null;
    },

    handle_response: async (response: Response) => {
        if (response.status == 200 || response.status == 201) {
            let data = await response.json();
            AuthAPI.current_user = {
                username: data['username'],
                email: data['email'],
                name: data['email'],
                user_id: data['user_id']
            };
            set_settings(data['settings']);
            
            events.emit('on_login');
        } else {
            let error = "Server error";
            try {
                error = await response.text();
            }
            catch (e) {
                console.error(e)
            }
            throw new Error(error);
        }
    }
};

export default AuthAPI;
