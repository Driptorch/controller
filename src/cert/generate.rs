use std::fmt;
use std::fmt::Formatter;
use picky::x509::{Cert, KeyIdGenMethod};
use picky::x509::certificate::{CertError, CertificateBuilder};
use picky::x509::date::UtcDate;

use chrono::prelude::*;
use picky::hash::HashAlgorithm;
use picky::key::PrivateKey;
use picky::signature::SignatureAlgorithm;
use picky::x509::name::DirectoryName;

pub enum InterTarget {
    CLIENT,
    PROXY
}

impl fmt::Display for InterTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InterTarget::CLIENT => write!(f, "CLIENT"),
            InterTarget::PROXY => write!(f, "PROXY")
        }
    }
}

pub async fn generate_root_cert(key: &PrivateKey) -> Result<Cert, CertError> {
    let current_date: DateTime<Utc> = Utc::now();

    CertificateBuilder::new()
        .validity(
            UtcDate::ymd(
                current_date.year() as u16,
                current_date.month() as u8,
                current_date.day() as u8
            ).unwrap(),
            UtcDate::ymd(
                (current_date.year() + 20) as u16,
                current_date.month() as u8,
                current_date.day() as u8
            ).unwrap()
        )
        .self_signed(DirectoryName::new_common_name("Driptorch"), key)
        .ca(true)
        .signature_hash_type(SignatureAlgorithm::RsaPkcs1v15(HashAlgorithm::SHA2_512))
        .key_id_gen_method(KeyIdGenMethod::SPKFullDER(HashAlgorithm::SHA2_512))
        .build()
}

pub async fn generate_inter_cert(key: &PrivateKey, target: InterTarget, root: (&Cert, &PrivateKey)) -> Result<Cert, CertError> {
    let current_date: DateTime<Utc> = Utc::now();

    CertificateBuilder::new()
        .validity(
            UtcDate::ymd(
                current_date.year() as u16,
                current_date.month() as u8,
                current_date.day() as u8
            ).unwrap(),
            UtcDate::ymd(
                (current_date.year() + 5) as u16,
                current_date.month() as u8,
                current_date.day() as u8
            ).unwrap()
        )
        .issuer_cert(&root.0, &root.1)
        .ca(true)
        .signature_hash_type(SignatureAlgorithm::RsaPkcs1v15(HashAlgorithm::SHA2_512))
        .key_id_gen_method(KeyIdGenMethod::SPKFullDER(HashAlgorithm::SHA2_512))
        .pathlen(0)
        .subject(
            DirectoryName::new_common_name(format!("Driptorch {}", target.to_string())),
            key.to_public_key()
        )
        .build()
}