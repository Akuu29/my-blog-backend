/// Configuration for Firebase JWKS endpoint used to verify Firebase ID tokens.
#[derive(Debug, Clone)]
pub struct FirebaseConfig {
    /// URL of the Firebase JWKS endpoint (e.g. `https://www.googleapis.com/...`).
    pub jwks_url: String,
}

impl FirebaseConfig {
    pub fn from_env() -> Self {
        Self {
            jwks_url: std::env::var("FIREBASE_JWKS_URL").expect("Undefined FIREBASE_JWKS_URL"),
        }
    }
}
