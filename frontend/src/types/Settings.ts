export interface Settings {
    common: {
        username: string;
    },
    mail: {
        address: string,
        username?: string,
        password?: string,
        from: string,
        to: string;
    };
    oidc: {
        id: string,
        secret: string,
        auth_url: string,
        callback_url: string;
    }
}
