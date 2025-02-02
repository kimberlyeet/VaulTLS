import { defineStore } from 'pinia';
import {is_setup, login} from "@/api/auth.ts";

export const useAuthStore = defineStore('auth', {
    state: () => ({
        token: '' as string | null,
        isAuthenticated: false as boolean,
        password_auth: false as boolean,
        oidc_url: null as string | null,
        error: null as string | null,
    }),
    actions: {
        async login(password: string | undefined) {
            try {
                this.token = (await login({password})).token;
                this.isAuthenticated = true;

                return true;
            } catch (err) {
                this.error = 'Failed to login.';
                console.error(err);
                return false;
            }
        },
        async is_setup() {
            try {
                const isSetupResponse = (await is_setup());
                this.password_auth = isSetupResponse.password;
                this.oidc_url = isSetupResponse.oidc;

                return isSetupResponse.setup;
            } catch (err) {
                this.error = 'Failed to login.';
                console.error(err);
                return false;
            }
        },
        logout() {
            this.token = null;
        },
    },
});
