import { defineStore } from 'pinia';
import type { Certificate } from '@/types/Certificate';
import {
    fetchCertificates,
    downloadCertificate,
    createCertificate,
    deleteCertificate,
} from '../api/certificates';
import type {CertificateRequirements} from "@/types/CertificateRequirements.ts"; // Adjust the path to match your project structure

export const useCertificateStore = defineStore('certificate', {
    state: () => ({
        certificates: [] as Certificate[], // Stores the list of certificates
        loading: false, // Indicates if an API call is in progress
        error: null as string | null, // Stores error messages
    }),

    actions: {
        // Fetch certificates and update the state
        async fetchCertificates(): Promise<void> {
            this.loading = true;
            this.error = null;
            try {
                this.certificates = await fetchCertificates();
            } catch (err) {
                this.error = 'Failed to fetch certificates.';
                console.error(err);
            } finally {
                this.loading = false;
            }
        },

        // Trigger the download of a certificate by ID
        async downloadCertificate(id: number): Promise<void> {
            try {
                const data = await downloadCertificate(id);
                const url = window.URL.createObjectURL(new Blob([data]));
                const link = document.createElement('a');
                link.href = url;
                link.setAttribute('download', `certificate-${id}.crt`);
                document.body.appendChild(link);
                link.click();
                link.remove();
                window.URL.revokeObjectURL(url);
            } catch (err) {
                this.error = 'Failed to download the certificate.';
                console.error(err);
            }
        },

        // Create a new certificate and fetch the updated list
        async createCertificate(certReq: CertificateRequirements): Promise<void> {
            this.loading = true;
            this.error = null;
            try {
                await createCertificate(certReq); // This will handle download and fetch internally
                this.certificates = await fetchCertificates(); // Refresh the local state
            } catch (err) {
                this.error = 'Failed to create the certificate.';
                console.error(err);
            } finally {
                this.loading = false;
            }
        },

        // Delete a certificate by ID and fetch the updated list
        async deleteCertificate(id: number): Promise<void> {
            this.loading = true;
            this.error = null;
            try {
                await deleteCertificate(id); // This handles API deletion and fetch internally
                this.certificates = await fetchCertificates(); // Refresh the local state
            } catch (err) {
                this.error = 'Failed to delete the certificate.';
                console.error(err);
            } finally {
                this.loading = false;
            }
        },
    },
});
