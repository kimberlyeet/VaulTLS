export interface SetupReq {
    name: string,
    ca_name: string,
    ca_validity_in_years: number,
    password: string | null;
}

export interface LoginReq {
    password: string;
}

export interface LoginResponse {
    token: string;
}

export interface IsSetupResponse {
    setup: boolean,
    password: boolean,
    oidc: string;
}