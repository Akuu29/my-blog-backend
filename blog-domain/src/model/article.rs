use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "article_status", rename_all = "lowercase")]
pub enum ArticleStatus {
    Draft,
    Published,
    Deleted,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub status: ArticleStatus,
    // pub user_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct NewArticle {
    pub title: String,
    pub body: String,
    pub status: ArticleStatus,
}

#[derive(Debug, Deserialize)]
pub struct UpdateArticle {
    pub title: Option<String>,
    pub body: Option<String>,
    pub status: Option<ArticleStatus>,
}
