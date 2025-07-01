export enum CertificateType {
    Client = 0,
    Server = 1,
    CA = 2
}

export interface Certificate {
    id: number;                         // Unique identifier for the certificate
    name: string;                       // Certificate name
    created_on: string;                 // Date when the certificate was created (UNIX timestamp in ms)
    pkcs12_password: string;            // PKCS12 decryption password
    valid_until: string;                // Expiration date of the certificate (UNIX timestamp in ms)
    certificate_type: CertificateType   // Type of the certificate
    user_id: number;                    // User ID who owns the certificate
}
