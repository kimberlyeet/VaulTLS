import { defineStore } from 'pinia';
import {login} from "@/api/auth.ts";

export const useAuthStore = defineStore('auth', {
    state: () => ({
        token: '' as string | null,
        isAuthenticated: false as boolean,
        error: null as string | null,
    }),
    actions: {
        async login(password: string) {
            try {
                this.token = (await login({password})).token;
                console.log("Token: " + this.token);
                this.isAuthenticated = true;

                return true;
            } catch (err) {
                this.error = 'Failed to setup.';
                console.error(err);
                return false;
            }
        },
        logout() {
            this.token = null;
        },
    },
});
