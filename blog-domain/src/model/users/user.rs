use serde::{Deserialize, Serialize};
use sqlx::{
    types::chrono::{DateTime, Local},
    FromRow,
};
use validator::Validate;

#[derive(Debug, Serialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
}

#[derive(Debug, Serialize, FromRow)]
pub struct User {
    id: i32,
    pub name: String,
    pub email: String,
    role: UserRole,
    idp_sub: String,
    created_at: DateTime<Local>,
    updated_at: DateTime<Local>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct NewUser {
    #[validate(length(min = 1, max = 255, message = "name length must be 1 to 255"))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 255, message = "sub length must be 1 to 255"))]
    pub idp_sub: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUser {
    #[validate(length(min = 1, max = 255, message = "name length must be 1 to 255"))]
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
}
