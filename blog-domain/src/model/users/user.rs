use crate::model::users::{email_cipher::EmailCipher, email_hash::EmailHash};
use rand::{Rng, distributions::Alphanumeric, thread_rng};
use serde::{Deserialize, Serialize};
use sqlx::{
    FromRow,
    types::{
        Uuid,
        chrono::{DateTime, Local},
    },
};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Default, Clone, PartialEq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    #[default]
    User,
}

#[allow(dead_code)]
#[derive(Debug, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "id")]
    pub public_id: Uuid,
    pub name: String,
    #[serde(skip)]
    pub role: UserRole,
    #[serde(skip)]
    pub is_active: bool,
    #[serde(skip)]
    last_login_at: Option<DateTime<Local>>,
    #[serde(skip)]
    created_at: DateTime<Local>,
    #[serde(skip)]
    updated_at: DateTime<Local>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct UserIdentity {
    pub provider_name: String,
    pub provider_user_id: String,
    pub provider_email_cipher: EmailCipher,
    pub provider_email_hash: EmailHash,
    pub provider_email_verified: bool,
    pub is_primary: bool,
}

#[derive(Debug)]
pub struct NewUser {
    pub name: String,
    pub role: UserRole,
    pub identity: UserIdentity,
}

impl NewUser {
    pub fn new(provider_name: &str, user_id: &str, email: &str, email_verified: bool) -> Self {
        let email_cipher = EmailCipher::from_plaintext(email);
        Self {
            name: Self::init_user_name(10),
            role: UserRole::default(),
            identity: UserIdentity {
                provider_name: provider_name.to_string(),
                provider_user_id: user_id.to_string(),
                provider_email_cipher: email_cipher,
                provider_email_hash: EmailHash::from_plaintext(email),
                provider_email_verified: email_verified,
                is_primary: true,
            },
        }
    }

    fn init_user_name(len: usize) -> String {
        let mut rng = thread_rng();
        let name: String = std::iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(len)
            .map(char::from)
            .collect();

        name
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUser {
    #[validate(length(min = 1, max = 15, message = "name length must be 1 to 15"))]
    pub name: Option<String>,
}
