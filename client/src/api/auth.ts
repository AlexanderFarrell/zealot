import { events } from "../core/events";

type User = {
    username: string;
    email: string;
    name: string;
    user_id: number;
}

export const AuthAPI = {
    current_user: null as User | null,

    login: async (username: string, password: string) => {
        const response = await fetch('/api/account/login', {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                username, 
                password
            })
        })
        await AuthAPI.handle_response(response);
    },

    register: async (username: string, password: string, 
        confirm: string, name: string, email: string) => {
        const response = await fetch('/api/account/register', {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                username,
                password,
                confirm,
                name,
                email
            })
        });
        await AuthAPI.handle_response(response);
    },

    logout: async (username: string, password: string) => {
        // Let the server know, whether successful or not.
        await fetch('/api/account/logout', {
            method: "GET",
        })
        events.emit('on_logout');
        AuthAPI.require_relogin();
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