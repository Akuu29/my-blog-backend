use serde::{Deserialize, Serialize};
use sqlx::{
    types::chrono::{DateTime, Local},
    FromRow,
};
use validator::Validate;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct NewTag {
    #[validate(length(min = 1, max = 15, message = "name length must be 1 to 15"))]
    pub name: String,
}
