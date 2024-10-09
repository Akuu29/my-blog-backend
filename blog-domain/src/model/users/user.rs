use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::PgRow,
    types::{
        chrono::{DateTime, Local},
        Uuid,
    },
    Error, FromRow, Row,
};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Default, Clone, PartialEq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    #[default]
    User,
}

#[derive(Debug, Serialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    idp_sub: String,
    created_at: DateTime<Local>,
    updated_at: DateTime<Local>,
}

impl<'r> FromRow<'r, PgRow> for User {
    fn from_row(row: &'r PgRow) -> Result<Self, Error> {
        let id: Uuid = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        let email: String = row.try_get("email")?;
        let role: UserRole = row.try_get("role")?;
        let idp_sub: String = row.try_get("idp_sub")?;
        let created_at: DateTime<Local> = row.try_get("created_at")?;
        let updated_at: DateTime<Local> = row.try_get("updated_at")?;

        Ok(User {
            id,
            name,
            email,
            role,
            idp_sub,
            created_at,
            updated_at,
        })
    }
}

#[derive(Debug, Default, Deserialize, Validate)]
pub struct NewUser {
    #[validate(length(min = 1, max = 255, message = "name length must be 1 to 255"))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 255, message = "sub length must be 1 to 255"))]
    pub idp_sub: String,
}

impl NewUser {
    pub fn new(&self, email: &str, idp_sub: &str) -> Self {
        Self {
            name: self.init_user_name(10),
            email: email.to_string(),
            idp_sub: idp_sub.to_string(),
        }
    }

    fn init_user_name(&self, len: usize) -> String {
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
    #[validate(length(min = 1, max = 255, message = "name length must be 1 to 255"))]
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
}
