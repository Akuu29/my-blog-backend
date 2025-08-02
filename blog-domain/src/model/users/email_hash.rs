use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::FromRow;
use std::env;

#[derive(Debug, Serialize, FromRow, Deserialize)]
pub struct EmailHash(pub Vec<u8>);

impl EmailHash {
    pub fn from_plaintext(plaintext: &str) -> Self {
        let pepper = env::var("EMAIL_HASH_PEPPER").expect("Undefined EMAIL_HASH_PEPPER");

        let mut mac = Hmac::<Sha256>::new_from_slice(pepper.as_bytes()).unwrap();
        mac.update(plaintext.as_bytes());
        let result = mac.finalize();
        let code_bytes = result.into_bytes().to_vec();

        Self(code_bytes)
    }
}
