use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use openssl::asn1::Asn1Time;
use openssl::bn::BigNum;
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::pkcs12::Pkcs12;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use openssl::stack::Stack;
use openssl::x509::{X509NameBuilder, X509};
use openssl::x509::extension::{BasicConstraints, KeyUsage, SubjectKeyIdentifier};
use openssl::x509::X509Builder;
use crate::ApiError;

#[derive(Default, Clone, rocket::serde::Serialize)]
pub struct Certificate {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) created_on: i64,
    pub(crate) valid_until: i64,
    #[serde(skip)]
    pub(crate) pkcs12: Vec<u8>,
    #[serde(skip)]
    pub(crate) cert: Vec<u8>,
    #[serde(skip)]
    pub(crate) key: Vec<u8>,
}

impl Certificate {
    pub fn set_id(&mut self, id: i64) -> () {
        self.id = id;
    }
}

pub fn create_ca(
    ca_name: &str,
    ca_validity_in_years: u64
) -> Result<Certificate, ErrorStack> {
    // Generate a private key
    let rsa = Rsa::generate(2048)?;
    let private_key = PKey::from_rsa(rsa)?;

    // Create the X509 name
    let mut name_builder = X509NameBuilder::new()?;
    name_builder.append_entry_by_text("CN", ca_name)?;
    let name = name_builder.build();

    // Create the certificate
    let mut builder = X509Builder::new()?;
    builder.set_version(2)?; // X509 v3
    builder.set_serial_number(&BigNum::from_u32(1)?.to_asn1_integer().unwrap())?;
    builder.set_subject_name(&name)?;
    builder.set_issuer_name(&name)?; // Self-signed
    builder.set_pubkey(&private_key)?;

    // Set the certificate validity
    let (created_on_unix, created_on_openssl) = get_timestamp(0)?;
    let (valid_until_unix, valid_until_openssl) = get_timestamp(ca_validity_in_years)?;
    builder.set_not_before(&created_on_openssl)?;
    builder.set_not_after(&valid_until_openssl)?;

    // Add basic constraints: This cert is a CA
    let basic_constraints = BasicConstraints::new().ca().build()?;
    builder.append_extension(basic_constraints)?;

    // Add key usage: Digital Signature and Key Cert Sign
    let key_usage = KeyUsage::new()
        .digital_signature()
        .key_cert_sign()
        .build()?;
    builder.append_extension(key_usage)?;

    // Add subject key identifier
    let subject_key_identifier = SubjectKeyIdentifier::new().build(&builder.x509v3_context(None, None))?;
    builder.append_extension(subject_key_identifier)?;

    // Sign the certificate with the private key
    builder.sign(&private_key, MessageDigest::sha256())?;

    // Build and return the private key and certificate
    let certificate = builder.build();
    
    Ok(Certificate{
        created_on: created_on_unix,
        valid_until: valid_until_unix,
        cert: certificate.to_der()?,
        key: private_key.private_key_to_der()?,
        ..Default::default()
    })
}

pub fn create_user_cert(
    ca: &Certificate,
    name: &str,
    validity_in_years: u64,
) -> Result<Certificate, ErrorStack> {
        let ca_cert = X509::from_der(&ca.cert)?;
        let ca_key = PKey::private_key_from_der(&ca.key)?;

        // Generate user's private key
        let rsa = Rsa::generate(4096)?;
        let user_key = PKey::from_rsa(rsa)?;
    
        // Create the user's X509 name (subject)
        let mut name_builder = X509NameBuilder::new()?;
        name_builder.append_entry_by_text("CN", name)?;
        let user_name = name_builder.build();
    
        // Create the user's certificate
        let mut user_cert_builder = X509::builder()?;
        user_cert_builder.set_version(2)?; // X.509 v3
        user_cert_builder.set_serial_number(&BigNum::from_u32(2)?.to_asn1_integer().unwrap())?;
        user_cert_builder.set_subject_name(&user_name)?;
        user_cert_builder.set_issuer_name(ca_cert.subject_name())?; // Signed by the CA
        user_cert_builder.set_pubkey(&user_key)?;
    
        // Set validity
        let (created_on_unix, created_on_openssl) = get_timestamp(0)?;
        let (valid_until_unix, valid_until_openssl) = get_timestamp(validity_in_years)?;
        user_cert_builder.set_not_before(&created_on_openssl)?;
        user_cert_builder.set_not_after(&valid_until_openssl)?;

        // Set key usage
        let key_usage = KeyUsage::new()
            .digital_signature()
            .build()?;
        user_cert_builder.append_extension(key_usage)?;
    
        let subject_key_identifier =
            SubjectKeyIdentifier::new().build(&user_cert_builder.x509v3_context(Some(&ca_cert), None))?;
        user_cert_builder.append_extension(subject_key_identifier)?;
    
        // Sign the user's certificate with the CA key
        user_cert_builder.sign(&ca_key, MessageDigest::sha256())?;
        let user_cert = user_cert_builder.build();
    
        let mut ca_stack = Stack::new()?;
        ca_stack.push(ca_cert.clone())?;
    
        // Create the PKCS#12 structure
        let pkcs12 = Pkcs12::builder()
            .name(&name)
            .ca(ca_stack)
            .cert(&user_cert)
            .pkey(&user_key)
            .build2("")?;

        Ok(Certificate{
            name: name.to_string(),
            created_on: created_on_unix,
            valid_until: valid_until_unix,
            pkcs12: pkcs12.to_der()?,
            ..Default::default()
        })
}

fn get_timestamp(from_now_in_years: u64) -> Result<(i64, Asn1Time), ErrorStack> {
    let time = SystemTime::now() + std::time::Duration::from_secs(60 * 60 * 24 * 365 * from_now_in_years);
    let time_unix = time.duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
    let time_openssl = Asn1Time::from_unix(time_unix)?;

    Ok((time_unix, time_openssl))
}

pub fn save_ca(ca: &Certificate) -> Result<(), ApiError> {
    let cert = X509::from_der(&ca.cert)?;
    let pem = cert.to_pem()?;
    fs::write("ca.cert", pem).map_err(|e| ApiError::Other(e.to_string()))?;
    Ok(())
}