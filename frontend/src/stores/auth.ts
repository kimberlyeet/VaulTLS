import { defineStore } from 'pinia';
import {change_password, current_user, is_setup, login} from "@/api/auth.ts";
import type {ChangePasswordReq} from "@/types/Login.ts";
import type {User} from "@/types/User.ts";

export const useAuthStore = defineStore('auth', {
    state: () => ({
        isInitialized: false as boolean,
        isSetup: false as boolean,
        isAuthenticated: false as boolean,
        password_auth: false as boolean,
        current_user: null as User | null,
        oidc_url: null as string | null,
        error: null as string | null,
    }),
    actions: {
        // Initializes auth store and fetches current user if authenticated
        async init() {
            this.error = null;
            await this.is_setup();
            this.isAuthenticated = localStorage.getItem('is_authenticated') === 'true';
            if (this.isAuthenticated) {
                await this.fetchCurrentUser();
            }
            this.isInitialized = true;
        },

        // Trigger the login of a user by email and password
        async login(email: string | undefined, password: string | undefined) {
            try {
                this.error = null;
                await login({email, password});
                this.current_user = (await current_user());
                this.setAuthentication(true);

                return true;
            } catch (err) {
                this.error = 'Failed to login.';
                console.error(err);
                return false;
            }
        },

        // Check if the app is set up and if a password is enabled
        async is_setup() {
            try {
                this.error = null;
                const isSetupResponse = (await is_setup());
                console.log(isSetupResponse);
                this.password_auth = isSetupResponse.password;
                this.oidc_url = isSetupResponse.oidc;
                this.isSetup = isSetupResponse.setup;
            } catch (err) {
                this.error = 'Failed to get setup state.';
                console.error(err);
                return false;
            }
        },

        // Change the password of the current user
        async change_password(changePasswordReq: ChangePasswordReq) {
            try {
                this.error = null;
                await change_password(changePasswordReq);
                this.password_auth = true;
                return true;
            } catch (err) {
                this.error = 'Failed to change password.';
                console.error(err);
                return false;
            }
        },

        // Fetch current user and update the state
        async fetchCurrentUser() {
            try {
                this.error = null;
                this.current_user = (await current_user());
                this.setAuthentication(true);
            } catch (err) {
                this.error = 'Failed to fetch current user.';
                console.error(err);
                this.logout();
            }
        },

        // Trigger the login of a user by OIDC
        async finishOIDC() {
            this.error = null;
            await this.fetchCurrentUser()
            this.setAuthentication(true);
        },

        // Set the authentication state and store it in local storage
        setAuthentication(isAuthenticated: boolean) {
            if (isAuthenticated) {
                this.isAuthenticated = true;
                localStorage.setItem('is_authenticated', String(true));
            } else {
                this.isAuthenticated = false;
                localStorage.removeItem('is_authenticated');
            }
        },

        // Logout the user and clear the authentication state
        logout() {
            this.error = null;
            this.setAuthentication(false);
        },
    },
});
