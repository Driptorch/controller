use std::{env, fmt, fs};
use std::fmt::Formatter;
use std::path::Path;
use chacha20poly1305::{AeadCore, KeyInit, XChaCha20Poly1305};
use chacha20poly1305::aead::{Aead, OsRng};

pub mod generate;

pub enum Types {
    ROOT,
    CLIENTINTER,
    PROXYINTER,
    CLIENTLEAF,
    PROXYLEAF
}

impl fmt::Display for Types {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Types::ROOT => write!(f, "ROOT"),
            Types::CLIENTINTER => write!(f, "CLIENTINTER"),
            Types::PROXYINTER => write!(f, "PROXYINTER"),
            Types::CLIENTLEAF => write!(f, "CLIENTLEAF"),
            Types::PROXYLEAF => write!(f, "PROXYLEAF")
        }
    }
}

pub async fn encrypt_priv_key(key: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    let cipher = XChaCha20Poly1305::new_from_slice(
        fs::read(
            Path::new(&env::var("XCC20_KEY").expect("XCC20_KEY must be set!"))
        ).expect("Failed to load the XCC20 key!").as_slice()
    ).expect("Error creating chacha20 cipher!");

    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);

    (nonce.to_vec(), cipher.encrypt(&nonce, key.as_slice()).expect("Failed to encrypt private key!"))
}