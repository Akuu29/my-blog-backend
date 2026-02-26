/// Configuration for JWT token generation and verification.
pub struct TokenConfig {
    /// HMAC-SHA256 signing key for access tokens.
    pub access_token_secret_key: String,
    /// HMAC-SHA256 signing key for refresh tokens.
    pub refresh_token_secret_key: String,
    /// Firebase project ID used as the expected audience when verifying Firebase ID tokens.
    pub firebase_project_id: String,
    /// Expected audience claim (`aud`) in access and refresh tokens.
    pub audience: String,
    /// Expected issuer claim (`iss`) in access and refresh tokens.
    pub issuer: String,
}

impl TokenConfig {
    pub fn from_env() -> Self {
        Self {
            access_token_secret_key: std::env::var("ACCESS_TOKEN_SECRET_KEY")
                .expect("Undefined ACCESS_TOKEN_SECRET_KEY"),
            refresh_token_secret_key: std::env::var("REFRESH_TOKEN_SECRET_KEY")
                .expect("Undefined REFRESH_TOKEN_SECRET_KEY"),
            firebase_project_id: std::env::var("FIREBASE_PROJECT_ID")
                .expect("Undefined FIREBASE_PROJECT_ID"),
            audience: std::env::var("AUDIENCE").expect("Undefined AUDIENCE"),
            issuer: std::env::var("ISSUER").expect("Undefined ISSUER"),
        }
    }
}

/// Configuration for image URL construction.
pub struct ImageConfig {
    /// Protocol part of the gateway URL (e.g. `https`).
    pub gateway_protocol: String,
    /// Domain part of the gateway URL (e.g. `example.com`).
    pub gateway_domain: String,
}

impl ImageConfig {
    pub fn from_env() -> Self {
        Self {
            gateway_protocol: std::env::var("GATEWAY_PROTOCOL")
                .expect("Undefined GATEWAY_PROTOCOL"),
            gateway_domain: std::env::var("GATEWAY_DOMAIN").expect("Undefined GATEWAY_DOMAIN"),
        }
    }
}
