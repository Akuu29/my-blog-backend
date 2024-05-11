use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupUser {
    pub email: String,
    pub password: String,
    #[serde(rename = "returnSecureToken")]
    pub return_secure_token: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
    #[serde(rename = "returnSecureToken")]
    pub return_secure_token: bool,
}
