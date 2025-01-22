import axios from 'axios';
import type { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';

class ApiClient {
    private client: AxiosInstance;

    constructor(baseURL: string) {
        this.client = axios.create({
            baseURL,
            headers: {
                'Content-Type': 'application/json',
            },
        });
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

export default new ApiClient('http://127.0.0.1:3737/'); // Your backend base URL
