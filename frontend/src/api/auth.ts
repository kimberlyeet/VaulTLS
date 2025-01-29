import type {SetupReq, LoginReq, LoginResponse} from "@/types/Login.ts";
import ApiClient from "@/api/ApiClient.ts";

export const is_setup = async (): Promise<boolean> => {
    return await ApiClient.get<boolean>('/is_setup');
};


export const setup = async (setupReq: SetupReq): Promise<void> => {
    return await ApiClient.post<void>('/setup', setupReq);
};

export const login = async (loginReq: LoginReq): Promise<LoginResponse> => {
    return await ApiClient.post<LoginResponse>('/auth/login', loginReq);
};