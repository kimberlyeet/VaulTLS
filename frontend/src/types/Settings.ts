export interface Settings {
    common: {
        username: string;
    };
    mail: {
        address: string;
        username?: string;
        password?: string;
        from: string;
        to: string;
    };
}
