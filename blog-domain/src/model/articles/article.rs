use serde::{Deserialize, Serialize};
use sqlx::{
    types::chrono::{DateTime, Local},
    FromRow,
};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "article_status", rename_all = "lowercase")]
pub enum ArticleStatus {
    Draft,
    Published,
    Deleted,
}

#[derive(Debug, Clone, Serialize, FromRow, PartialEq)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub status: ArticleStatus,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    // pub user_id: i32,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct NewArticle {
    #[validate(length(min = 1, max = 85, message = "title length must be 1 to 85"))]
    pub title: String,
    #[validate(length(min = 1, message = "body length mut be 1 or more"))]
    pub body: String,
    pub status: ArticleStatus,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateArticle {
    #[validate(length(min = 1, max = 85, message = "title length must be 1 to 85"))]
    pub title: Option<String>,
    #[validate(length(min = 1, message = "body length mut be 1 or more"))]
    pub body: Option<String>,
    pub status: Option<ArticleStatus>,
}
