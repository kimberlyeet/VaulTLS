import { defineStore } from 'pinia';
import {type Settings} from '@/types/Settings';
import {
    fetchSettings, fetchVersion,
    putSettings
} from '@/api/settings';

export const useSettingsStore = defineStore('settings', {
    state: () => ({
        settings: null as Settings | null,
        version: null as string | null,
        error: null as string | null,
    }),

    actions: {
        // Fetch server version from API
        async fetchVersion(): Promise<void> {
            this.error = null;
            try {
                this.version = await fetchVersion();
            } catch (err) {
                this.error = 'Failed to fetch server version.';
                console.error(err);
            }
        },

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
            if (this.settings) {
                try {
                    this.error = null;
                    await putSettings(this.settings);
                    return true;
                } catch (err) {
                    this.error = 'Failed to download the certificate.';
                    console.error(err);
                    return false;
                }
            }
            return false;
        },
    },
});
