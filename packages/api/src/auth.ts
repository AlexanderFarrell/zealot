import {account} from "@zealot/domain";
import {get_req, post_req} from "@websoil/engine";
import { BaseAPI } from "./common";
import { Account } from "@zealot/domain/src/account";

export class AuthAPI extends BaseAPI {
    public Account: Account | null;
    public Basic: BasicAuthAPI;

    public constructor(baseURL: string) {
        let onLogin = (dto: account.AccountDto) => {
            this.Account = new Account(dto);
        }

        super(baseURL)
        this.Basic = new BasicAuthAPI(baseURL, onLogin);
        this.Account = null;
    }

    public async IsLoggedIn() {
        if (this.Account != null) {
            return false;
        }

        try {
            const response = await get_req(`${this.baseUrl}/account/is_logged_in`);
            return response.ok || response.status == 201;
        } catch (e) {
            return false;
        }
    }
}

class BasicAuthAPI extends BaseAPI {
    private onLogin: (dto: account.AccountDto) => void;

    public constructor(baseUrl: string, onLogin: (dto: account.AccountDto) => void)  {
        super(baseUrl)
        this.onLogin = onLogin;
    }

    async login(dto: account.LoginBasicDto) {
        const response = await post_req(`${this.baseUrl}/account/login`, {
            username: dto.username,
            password: dto.password
        })  
        this.onLoginAttempt(response);        
    }

    async register(dto: account.RegisterBasicDto) {
        if (dto.password != dto.confirm) {
            throw new Error("Passwords do not match.");
        }
        const response = await post_req(`${this.baseUrl}/account/register`, {
            username: dto.username,
            password: dto.password,
            confirm: dto.confirm,
            email: dto.email,
            given_name: dto.given_name,
            surname: dto.surname
        })
        this.onLoginAttempt(response)
    }

    async logout() {
        await get_req(`${this.baseUrl}/account/logout`);
    }

    private async onLoginAttempt(response: Response) {
        if (response.status == 200 || response.status == 201) {
            let data = await response.json();
            this.onLogin({
                account_id: data['account_id'],
                username: data['username'],
                email: data['email'],
                given_name: data['given_name'],
                surname: data['surname']
            })
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
}

