import { get_json, post_json, post_req } from "@websoil/engine";
import { BaseAPI } from "./common";
import { Account, type AccountDto, type LoginBasicDto, type RegisterBasicDto } from "@zealot/domain/src/account";

export class AuthAPI extends BaseAPI {
    public Account: Account | null;
    public Basic: BasicAuthAPI;

    public constructor(baseURL: string) {
        super(baseURL);
        this.Account = null;
        this.Basic = new BasicAuthAPI(baseURL,
            (dto) => { this.Account = new Account(dto); },
            ()    => { this.Account = null; }
        );
    }

    public async IsLoggedIn(): Promise<boolean> {
        if (this.Account != null) {
            return true;
        }
        try {
            const dto = await get_json(`${this.baseUrl}/auth/is_logged_in`) as AccountDto;
            this.Account = new Account(dto);
            return true;
        } catch {
            return false;
        }
    }
}

class BasicAuthAPI extends BaseAPI {
    private onLogin: (dto: AccountDto) => void;
    private onLogout: () => void;

    public constructor(
        baseUrl: string,
        onLogin: (dto: AccountDto) => void,
        onLogout: () => void,
    ) {
        super(baseUrl);
        this.onLogin = onLogin;
        this.onLogout = onLogout;
    }

    public async login(dto: LoginBasicDto): Promise<Account> {
        const data = await post_json(`${this.baseUrl}/auth/login`, {
            username: dto.username,
            password: dto.password,
        }) as AccountDto;
        this.onLogin(data);
        return new Account(data);
    }

    public async register(dto: RegisterBasicDto): Promise<Account> {
        if (dto.password !== dto.confirm) {
            throw new Error("Passwords do not match.");
        }
        const data = await post_json(`${this.baseUrl}/auth/register`, {
            username: dto.username,
            password: dto.password,
            confirm: dto.confirm,
            email: dto.email,
            given_name: dto.given_name,
            surname: dto.surname,
        }) as AccountDto;
        this.onLogin(data);
        return new Account(data);
    }

    public async logout(): Promise<void> {
        await post_req(`${this.baseUrl}/auth/logout`, {});
        this.onLogout();
    }
}
