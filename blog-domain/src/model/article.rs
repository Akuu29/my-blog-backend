use serde::{Deserialize, Serialize};
use sqlx::{
    types::chrono::{DateTime, Local},
    FromRow,
};

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

#[derive(Debug, Clone, Deserialize)]
pub struct NewArticle {
    pub title: String,
    pub body: String,
    pub status: ArticleStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateArticle {
    pub title: Option<String>,
    pub body: Option<String>,
    pub status: Option<ArticleStatus>,
}
