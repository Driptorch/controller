pub mod auth;

use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::SaltString;
use base64ct::{Base64UrlUnpadded, Encoding};
use rand::{rngs::OsRng, RngCore};

pub fn generate_session_token() -> String {
    let mut randombytes = [0u8; 32];
    OsRng.fill_bytes(&mut randombytes);

    let session_token = Base64UrlUnpadded::encode_string(blake3::hash(&randombytes).as_bytes());

    session_token
}

pub fn hash_password(password: String) -> String {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    argon2.hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password!")
        .to_string()
}