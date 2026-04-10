

export class Account {
    public readonly AccountID: number;
    public Username: string;
    public Email: string;
    public GivenName: string;
    public Surname: string;

    public constructor(dto: AccountDto) {
        this.AccountID = dto.account_id;
        this.Username = dto.username;
        this.Email = dto.email;
        this.GivenName = dto.given_name;
        this.Surname = dto.surname;
    }

    public get FullNameEng(): string {
        return `${this.GivenName} ${this.Surname}`;
    }

    public get FullNameFormalEng(): string {
        return `${this.Surname}, ${this.GivenName}`
    }
}

export interface AccountDto {
    account_id: number;
    username: string;
    email: string;
    given_name: string;
    surname: string;
    settings: Record<string, unknown>;
}

export interface LoginBasicDto {
    username: string;
    password: string;
}

export interface RegisterBasicDto {
    username: string;
    password: string;
    confirm: string;
    email: string;
    given_name: string;
    surname: string;
}
