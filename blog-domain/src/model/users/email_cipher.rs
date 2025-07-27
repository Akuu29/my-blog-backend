use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use serde::Serialize;
use sqlx::{FromRow, types::Json};
use std::env;

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
    pub fn from_plaintext(plaintext: &str) -> Self {
        let (ciphertext, nonce) = Self::encrypt_email(plaintext);

        Self {
            ciphertext,
            nonce: nonce,
            meta: Json(CipherMetadata::default()),
        }
    }

    fn encrypt_email(email: &str) -> (Vec<u8>, Vec<u8>) {
        let encryption_key = env::var("ENCRYPTION_KEY").expect("Undefined ENCRYPTION_KEY");
        let key = Key::<Aes256Gcm>::from_slice(encryption_key.as_bytes());

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let cipher = Aes256Gcm::new(key);
        let ciphertext = cipher.encrypt(&nonce, email.as_bytes()).unwrap();
        (ciphertext, nonce.to_vec())
    }

    pub fn decrypt_email(&self, ciphertext: &[u8], nonce: &[u8]) -> String {
        let encryption_key = env::var("ENCRYPTION_KEY").expect("Undefined ENCRYPTION_KEY");
        let key = Key::<Aes256Gcm>::from_slice(encryption_key.as_bytes());

        let nonce = Nonce::from_slice(nonce);

        let cipher = Aes256Gcm::new(key);
        let plaintext = cipher.decrypt(nonce, ciphertext).unwrap();
        String::from_utf8(plaintext).unwrap()
    }
}
