import ApiClient from './ApiClient';
import type { Certificate } from '@/types/Certificate';
import type {CertificateRequirements} from "@/types/CertificateRequirements.ts";

export const fetchCertificates = async (): Promise<Certificate[]> => {
    return await ApiClient.get<Certificate[]>('/certificates');
};

export const downloadCertificate = async (id: number): Promise<Blob> => {
    return await ApiClient.get<Blob>(`/certificates/${id}/download`, { responseType: 'blob' });
};

export const createCertificate = async (certReq: CertificateRequirements): Promise<number> => {
    const cert = await ApiClient.post<Certificate>('/certificates', certReq);
    return cert.id;
};

export const handleDeleteCertificate = async (id: number): Promise<void> => {
    await ApiClient.delete<void>(`/certificates/${id}`);
    await fetchCertificates();
};
