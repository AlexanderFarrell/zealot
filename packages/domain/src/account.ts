

export class User {
    public readonly UserID: number;
    public Username: string;
    public Email: string;
    public GivenName: string;
    public Surname: string;

    public constructor(dto: UserDto) {
        this.UserID = dto.user_id;
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

export interface UserDto {
    user_id: number;
    username: string;
    email: string;
    given_name: string;
    surname: string;
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
