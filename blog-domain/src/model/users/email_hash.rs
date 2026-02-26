use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow, Deserialize)]
pub struct EmailHash(pub Vec<u8>);

impl EmailHash {
    pub fn from_plaintext(plaintext: &str, pepper: &str) -> Self {
        let mut mac = Hmac::<Sha256>::new_from_slice(pepper.as_bytes()).unwrap();
        mac.update(plaintext.as_bytes());
        let result = mac.finalize();
        let code_bytes = result.into_bytes().to_vec();

        Self(code_bytes)
    }
}
