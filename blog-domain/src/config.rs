/// Configuration for email encryption and hashing.
#[derive(Clone)]
pub struct EmailConfig {
    /// HMAC pepper used for hashing email addresses.
    pub pepper: String,
    /// AES-256-GCM encryption key for storing email addresses.
    pub encryption_key: String,
}

impl EmailConfig {
    pub fn from_env() -> Self {
        Self {
            pepper: std::env::var("EMAIL_HASH_PEPPER").expect("Undefined EMAIL_HASH_PEPPER"),
            encryption_key: std::env::var("EMAIL_ENCRYPTION_KEY")
                .expect("Undefined EMAIL_ENCRYPTION_KEY"),
        }
    }
}
