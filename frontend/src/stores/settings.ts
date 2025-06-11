import { defineStore } from 'pinia';
import {Encryption, type Settings} from '@/types/Settings';
import {
    fetchSettings,
    putSettings
} from '@/api/settings';

const defaultSettings: Settings = {
    common: {
        password_enabled: false,
        vaultls_url: '',
    },
    mail: {
        smtp_host: '',
        smtp_port: 0,
        encryption: Encryption.None,
        username: undefined,
        password: undefined,
        from: '',
    },
    oidc: {
        id: '',
        secret: '',
        auth_url: '',
        callback_url: '',
    }
};

export const useSettingsStore = defineStore('settings', {
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
        async saveSettings(): Promise<boolean> {
            try {
                this.error = null;
                await putSettings(this.settings);
                return true;
            } catch (err) {
                this.error = 'Failed to download the certificate.';
                console.error(err);
                return false;
            }
        },
    },
});
