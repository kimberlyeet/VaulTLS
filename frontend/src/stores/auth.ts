import { defineStore } from 'pinia';
import {change_password, is_setup, login} from "@/api/auth.ts";
import type {ChangePasswordReq} from "@/types/Login.ts";

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
                this.error = 'Failed to get setup state.';
                console.error(err);
                return false;
            }
        },
        async change_password(changePasswordReq: ChangePasswordReq) {
            try {
                await change_password(changePasswordReq);
                this.password_auth = true;
                return true;
            } catch (err) {
                this.error = 'Failed to change password.';
                console.error(err);
                return false;
            }
        },
        logout() {
            this.token = null;
        },
    },
});
