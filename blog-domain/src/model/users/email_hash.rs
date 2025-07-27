use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow, Deserialize)]
pub struct EmailHash(pub Vec<u8>);

impl EmailHash {
    pub fn from_plaintext(plaintext: &str) -> Self {
        let email_hash = Sha256::digest(plaintext.as_bytes());

        Self(email_hash.to_vec())
    }
}
