use serde::{Deserialize, Serialize};
use sqlx::{
    types::chrono::{DateTime, Local},
    FromRow,
};
use validator::Validate;

#[derive(Debug, Serialize, FromRow)]
pub struct Category {
    pub id: i32,
    pub name: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Local>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct NewCategory {
    #[validate(length(min = 1, max = 20, message = "category length must be 1 to 20"))]
    pub name: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCategory {
    #[validate(length(min = 1, max = 20, message = "category length must be 1 to 20"))]
    pub name: String,
}
