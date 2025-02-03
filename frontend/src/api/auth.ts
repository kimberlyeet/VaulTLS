import type {SetupReq, LoginResponse, IsSetupResponse, ChangePasswordReq} from "@/types/Login.ts";
import ApiClient from "@/api/ApiClient.ts";

export const is_setup = async (): Promise<IsSetupResponse> => {
    return await ApiClient.get<IsSetupResponse>('/is_setup');
};


export const setup = async (setupReq: SetupReq): Promise<void> => {
    return await ApiClient.post<void>('/setup', setupReq);
};

export const login = async (loginReq: { password: string | undefined }): Promise<LoginResponse> => {
    return await ApiClient.post<LoginResponse>('/auth/login', loginReq);
};

export const change_password = async (changePasswordReq: ChangePasswordReq): Promise<void> => {
    return await ApiClient.post<void>('/auth/change_password', changePasswordReq);
};