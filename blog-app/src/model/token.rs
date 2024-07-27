use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct IdTokenClaims {
    exp: usize,
    iat: usize,
    aud: String,
    iss: String,
    sub: String,
    auth_time: usize,
}

impl IdTokenClaims {
    pub fn sub(&self) -> String {
        self.sub.to_string()
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AccessTokenClaims {
    exp: usize,
    iat: usize,
    aud: String,
    iss: String,
    sub: String,
    /// the time at which the token was valid
    nbf: usize,
    /// unique identifier for the token
    jti: String,
    /// user roles
    roles: Vec<String>,
}

impl AccessTokenClaims {
    pub fn new(sub: String) -> Self {
        let now = Utc::now();
        let expiration = now + chrono::Duration::hours(1);
        let not_before = now;

        AccessTokenClaims {
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
            aud: std::env::var("SERVICE_NAME").expect("undefined API_NAME"),
            iss: std::env::var("API_NAME").expect("undefined API_NAME"),
            sub: sub,
            nbf: not_before.timestamp() as usize,
            jti: uuid::Uuid::new_v4().to_string(),
            roles: vec![
                "reader".to_string(),
                "writer".to_string(),
                "admin".to_string(),
            ],
        }
    }

    pub fn sub(&self) -> String {
        self.sub.to_string()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RefreshTokenClaims {
    exp: usize,
    iat: usize,
    aud: String,
    iss: String,
    sub: String,
}

impl RefreshTokenClaims {
    pub fn new(sub: String) -> Self {
        let now = Utc::now();
        let expiration = now + chrono::Duration::days(30);

        RefreshTokenClaims {
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
            aud: std::env::var("SERVICE_NAME").expect("undefined SERVICE_NAME"),
            iss: std::env::var("API_NAME").expect("undefined API_NAME"),
            sub: sub,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiCredentials {
    access_token: String,
    refresh_token: String,
    // pub expires_in: String,
    // local_id: String,
}

impl ApiCredentials {
    pub fn new(access_token: String, refresh_token: String) -> Self {
        Self {
            access_token,
            refresh_token,
        }
    }
}
