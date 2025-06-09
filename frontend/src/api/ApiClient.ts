import axios from 'axios';
import type { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';
import {useAuthStore} from "@/stores/auth.ts";
const API_URL = `${window.location.origin}/api`;

class ApiClient {
    private client: AxiosInstance;

    constructor(baseURL: string) {
        this.client = axios.create({
            baseURL,
            headers: {
                'Content-Type': 'application/json',
            },
        });

        this.client.interceptors.request.use(
            (config) => {
                const authStore = useAuthStore();
                const token = authStore.token;

                if (token) {
                    config.headers.Authorization = `Bearer ${token}`;
                }

                return config;
            },
            (error) => {
                return Promise.reject(error);
            }
        );

        // Response interceptor to handle token expiration
        this.client.interceptors.response.use(
            (response) => response,
            async (error) => {
                if (error.response && error.response.status === 401) {
                    const authStore = useAuthStore();
                    authStore.logout();
                    window.location.href = '/';
                }
                return Promise.reject(error);
            }
        );
    }

    async get<T>(url: string, params: Record<string, any> = {}): Promise<T> {
        try {
            const response: AxiosResponse<T> = await this.client.get(url, { params });
            return response.data;
        } catch (error) {
            console.error(`GET ${url} failed:`, error);
            throw error;
        }
    }

    async post<T>(url: string, data: Record<string, any> = {}): Promise<T> {
        try {
            const response: AxiosResponse<T> = await this.client.post(url, data);
            return response.data;
        } catch (error) {
            console.error(`POST ${url} failed:`, error);
            throw error;
        }
    }

    async put<T>(url: string, data: Record<string, any> = {}): Promise<T> {
        try {
            const response: AxiosResponse<T> = await this.client.put(url, data);
            return response.data;
        } catch (error) {
            console.error(`PUT ${url} failed:`, error);
            throw error;
        }
    }

    async delete<T>(url: string, config: AxiosRequestConfig = {}): Promise<T> {
        try {
            const response: AxiosResponse<T> = await this.client.delete(url, config);
            return response.data;
        } catch (error) {
            console.error(`DELETE ${url} failed:`, error);
            throw error;
        }
    }
}

export default new ApiClient(API_URL);
