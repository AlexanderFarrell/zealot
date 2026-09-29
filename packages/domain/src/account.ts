export interface ApiKeyDto {
    api_key_id: number;
    label: string;
    created_at: string;
}

export class ApiKey {
    public readonly ApiKeyId: number;
    public readonly Label: string;
    public readonly CreatedAt: string;

    public constructor(dto: ApiKeyDto) {
        this.ApiKeyId = dto.api_key_id;
        this.Label = dto.label;
        this.CreatedAt = dto.created_at;
    }
}

export interface CreateApiKeyResponseDto {
    key: string;
    api_key_id: number;
    label: string;
    created_at: string;
}

export class Account {
    public readonly AccountID: number;
    public Username: string;
    public Email: string;
    public GivenName: string;
    public Surname: string;
    public HasApiKey: boolean;

    public constructor(dto: AccountDto) {
        this.AccountID = dto.account_id;
        this.Username = dto.username;
        this.Email = dto.email;
        this.GivenName = dto.given_name;
        this.Surname = dto.surname;
        this.HasApiKey = dto.has_api_key;
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
    has_api_key: boolean;
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
