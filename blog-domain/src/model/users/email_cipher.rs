use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::{FromRow, types::Json};

#[derive(Debug, Serialize)]
pub struct CipherMetadata {
    pub alg: String,
    pub key_ver: i32,
}

impl CipherMetadata {
    pub fn new(alg: &str, key_ver: i32) -> Self {
        Self {
            alg: alg.to_string(),
            key_ver,
        }
    }
}

impl Default for CipherMetadata {
    fn default() -> Self {
        Self {
            alg: "aes-256-gcm".to_string(),
            key_ver: 1,
        }
    }
}

#[derive(Debug, Serialize, FromRow)]
pub struct EmailCipher {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub meta: Json<CipherMetadata>,
}

impl EmailCipher {
    pub fn from_plaintext(plaintext: &str, encryption_key: &str) -> anyhow::Result<Self> {
        let (ciphertext, nonce) = Self::encrypt_email(plaintext, encryption_key)?;

        Ok(Self {
            ciphertext,
            nonce,
            meta: Json(CipherMetadata::default()),
        })
    }

    fn derive_key(encryption_key: &str) -> anyhow::Result<Key<Aes256Gcm>> {
        let mut hasher = Sha256::default();
        hasher.update(encryption_key.as_bytes());
        let key_bytes = hasher.finalize();

        Ok(*Key::<Aes256Gcm>::from_slice(&key_bytes))
    }

    fn encrypt_email(email: &str, encryption_key: &str) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
        let key = Self::derive_key(encryption_key)?;
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let cipher = Aes256Gcm::new(&key);
        let ciphertext = cipher
            .encrypt(&nonce, email.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to encrypt email: {}", e))?;
        Ok((ciphertext, nonce.to_vec()))
    }

    pub fn decrypt_email(
        &self,
        ciphertext: &[u8],
        nonce: &[u8],
        encryption_key: &str,
    ) -> anyhow::Result<String> {
        let key = Self::derive_key(encryption_key)?;
        let nonce = Nonce::from_slice(nonce);

        let cipher = Aes256Gcm::new(&key);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Failed to decrypt email: {}", e))?;
        String::from_utf8(plaintext).map_err(|e| anyhow::anyhow!("Invalid UTF-8: {}", e))
    }
}
