import ApiClient from './ApiClient';
import type {Settings} from "@/types/Settings.ts";

export const fetchSettings = async (): Promise<Settings> => {
    return await ApiClient.get<Settings>('/settings');
};

export const putSettings = async (settings: Settings): Promise<void> => {
    return await ApiClient.put<void>(`/settings`, settings);
};
