export interface SetupReq {
    name: string,
    ca_name: string,
    ca_validity_in_years: number,
    password: string;
}

export interface LoginReq {
    password: string;
}

export interface LoginResponse {
    token: string;
}