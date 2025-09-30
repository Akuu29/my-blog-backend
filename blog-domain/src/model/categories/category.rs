use serde::{Deserialize, Serialize};
use sqlx::{
    FromRow,
    types::chrono::{DateTime, Local},
};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    #[serde(rename = "id")]
    pub public_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct NewCategory {
    #[validate(length(min = 1, max = 20, message = "category length must be 1 to 20"))]
    pub name: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCategory {
    #[validate(length(min = 1, max = 20, message = "category length must be 1 to 20"))]
    pub name: String,
}
