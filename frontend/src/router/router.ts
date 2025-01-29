import { createRouter, createWebHistory } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import { is_setup } from '@/api/auth';

import LoginView from '@/views/LoginView.vue';
import FirstSetupView from '@/views/FirstSetupView.vue';

import MainLayout from '@/layouts/MainLayout.vue';
import OverviewTab from '@/components/OverviewTab.vue';
import GenerateTab from '@/components/GenerateTab.vue';
import SettingsTab from '@/components/SettingsTab.vue';

const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: '/login',
            name: 'Login',
            component: LoginView,
        },
        {
            path: '/first-setup',
            name: 'FirstSetup',
            component: FirstSetupView,
        },
        {
            path: '/',
            component: MainLayout,
            // Child routes for the main app
            children: [
                {
                    path: '',
                    redirect: '/overview', // default child route
                },
                {
                    path: 'overview',
                    name: 'Overview',
                    component: OverviewTab,
                },
                {
                    path: 'generate',
                    name: 'Generate',
                    component: GenerateTab,
                },
                {
                    path: 'settings',
                    name: 'Settings',
                    component: SettingsTab,
                },
            ],
            // A guard to check if the app is set up and user is authenticated
            beforeEnter: async (to, from, next) => {
                const authStore = useAuthStore();
                try {
                    const is_backend_setup = await is_setup();
                    console.log("Is setup: " + is_backend_setup);
                    if (!is_backend_setup) {
                        return next({ name: 'FirstSetup' });
                    }

                    console.log("Is authenticated: " + authStore.isAuthenticated);
                    if (!authStore.isAuthenticated) {
                        return next({ name: 'Login' });
                    }

                    // Otherwise, proceed
                    next();
                } catch (error) {
                    console.error('Error checking setup or auth:', error);
                    next({ name: 'Login' });
                }
            },
        },
    ],
});

export default router;
