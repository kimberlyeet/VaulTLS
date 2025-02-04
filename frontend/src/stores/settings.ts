import { defineStore } from 'pinia';
import type { Settings } from '@/types/Settings';
import {
    fetchSettings,
    putSettings
} from '@/api/settings'; // Adjust the path to match your project structure

const defaultSettings: Settings = {
    common: {
        username: '',
    },
    mail: {
        address: '',
        username: undefined,
        password: undefined,
        from: '',
        to: '',
    },
    oidc: {
        id: '',
        secret: '',
        auth_url: '',
        callback_url: '',
    }
};

export const useSettingseStore = defineStore('settings', {
    state: () => ({
        settings: defaultSettings,
        error: null as string | null,
    }),

    actions: {
        // Fetch certificates and update the state
        async fetchSettings(): Promise<void> {
            this.error = null;
            try {
                this.settings = await fetchSettings();
            } catch (err) {
                this.error = 'Failed to fetch settings.';
                console.error(err);
            }
        },

        // Trigger the download of a certificate by ID
        async saveSettings(): Promise<void> {
            try {
                await putSettings(this.settings);
            } catch (err) {
                this.error = 'Failed to download the certificate.';
                console.error(err);
            }
        },
    },
});
