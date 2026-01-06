use crate::model::users::user::{User, UserRole};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use url::Url;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct IdTokenClaims {
    exp: usize,
    iat: usize,
    aud: String,
    iss: String,
    sub: String,
    auth_time: usize,
    user_id: String,
    email: String,
    email_verified: bool,
}

impl IdTokenClaims {
    pub fn sub(&self) -> String {
        self.sub.clone()
    }
    pub fn email(&self) -> String {
        self.email.clone()
    }
    pub fn email_verified(&self) -> bool {
        self.email_verified
    }
    pub fn provider_name(&self) -> anyhow::Result<String> {
        let url = Url::parse(&self.iss)?;
        match url.host_str() {
            Some("securetoken.google.com") => Ok("firebase".to_string()),
            Some(host) => Err(anyhow::anyhow!("Unsupported provider: {}", host)),
            None => Err(anyhow::anyhow!("Invalid issuer URL: no host found")),
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct AccessTokenClaims {
    exp: usize,
    iat: usize,
    aud: String,
    iss: String,
    sub: Uuid,
    /// the time at which the token was valid
    nbf: usize,
    /// unique identifier for the token
    jti: String,
    pub role: UserRole,
}

impl AccessTokenClaims {
    pub fn new(user: &User) -> Self {
        let now = Utc::now();
        let expiration = now + chrono::Duration::hours(1);
        let not_before = now;

        AccessTokenClaims {
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
            aud: std::env::var("AUDIENCE").expect("undefined AUDIENCE"),
            iss: std::env::var("ISSUER").expect("undefined ISSUER"),
            sub: user.public_id,
            nbf: not_before.timestamp() as usize,
            jti: uuid::Uuid::new_v4().to_string(),
            role: user.role.clone(),
        }
    }

    pub fn sub(&self) -> Uuid {
        self.sub
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RefreshTokenClaims {
    exp: usize,
    iat: usize,
    aud: String,
    iss: String,
    sub: Uuid,
}

impl RefreshTokenClaims {
    pub fn new(user: &User) -> Self {
        let now = Utc::now();
        let expiration = now + chrono::Duration::days(30);

        RefreshTokenClaims {
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
            aud: std::env::var("AUDIENCE").expect("undefined AUDIENCE"),
            iss: std::env::var("ISSUER").expect("undefined ISSUER"),
            sub: user.public_id,
        }
    }

    pub fn sub(&self) -> Uuid {
        self.sub
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiCredentials {
    access_token: String,
    user: User,
}

impl ApiCredentials {
    pub fn new(access_token: &str, user: User) -> Self {
        Self {
            access_token: access_token.to_string(),
            user,
        }
    }
}
