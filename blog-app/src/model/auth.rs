use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize)]
pub struct UserCredentials {
    pub kind: String,
    #[serde(rename = "idToken")]
    pub id_token: String,
    pub email: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
    #[serde(rename = "expiresIn")]
    pub expires_in: String,
    #[serde(rename = "localId")]
    pub local_id: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SignupUser {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6, message = "password length must be 6 or more"))]
    pub password: String,
    #[serde(rename = "returnSecureToken")]
    pub return_secure_token: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SigninUser {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6, message = "password length must be 6 or more"))]
    pub password: String,
    #[serde(rename = "returnSecureToken")]
    pub return_secure_token: bool,
}
