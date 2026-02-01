use serde::{Deserialize, Serialize};
use sqlx::{
    FromRow,
    types::chrono::{DateTime, Local},
};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, FromRow, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    #[serde(rename = "id")]
    pub public_id: Uuid,
    #[serde(rename = "userId")]
    pub user_public_id: Option<Uuid>,
    pub user_name: Option<String>,
    #[serde(rename = "articleId")]
    pub article_public_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct NewComment {
    #[serde(rename = "articleId")]
    pub article_public_id: Uuid,
    #[validate(length(min = 1, message = "body length must be 1 or more"))]
    pub body: String,
    // Only for guest users; logged-in users will have their ID extracted from access token
    pub user_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateComment {
    #[validate(length(min = 1, message = "body length must be 1 or more"))]
    pub body: Option<String>,
}
