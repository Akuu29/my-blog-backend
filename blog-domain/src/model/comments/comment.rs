use serde::{Deserialize, Serialize};
use sqlx::{
    types::chrono::{DateTime, Local},
    FromRow,
};
use validator::Validate;

#[derive(Debug, Clone, Serialize, FromRow, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: i32,
    pub article_id: i32,
    pub body: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct NewComment {
    pub article_id: i32,
    #[validate(length(min = 1, message = "body length must be 1 or more"))]
    pub body: String,
    pub user_id: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateComment {
    #[validate(length(min = 1, message = "body length must be 1 or more"))]
    pub body: Option<String>,
}
