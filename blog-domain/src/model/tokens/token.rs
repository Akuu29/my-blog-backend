use crate::model::users::user::{User, UserRole};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

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
}

impl IdTokenClaims {
    pub fn sub(&self) -> String {
        self.sub.clone()
    }
    pub fn email(&self) -> String {
        self.email.clone()
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
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
            sub: user.id,
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
            sub: user.id,
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
}

impl ApiCredentials {
    pub fn new(access_token: &str) -> Self {
        Self {
            access_token: access_token.to_string(),
        }
    }
}
